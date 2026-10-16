//! Native Windows host mechanisms. Handles are scoped to the operation owning them.
use std::{ffi::OsStr,os::windows::{ffi::OsStrExt,io::AsRawHandle,process::CommandExt},path::Path,process::{Child,Command}};
use windows_sys::Win32::{Foundation::*,System::{SystemInformation::*,Threading::*,JobObjects::*},Security::{*,Authorization::*}};
fn error()->String {std::io::Error::last_os_error().to_string()}
fn wide(value:impl AsRef<OsStr>)->Result<Vec<u16>,String> {
    let mut value=value.as_ref().encode_wide().collect::<Vec<_>>();
    if value.contains(&0){return Err("Invalid system path".into());}value.push(0);Ok(value)
}
pub fn open_external(target:impl AsRef<OsStr>)->Result<(),String> {
    let target=wide(target)?;let verb=wide("open")?;
    let result=unsafe {windows_sys::Win32::UI::Shell::ShellExecuteW(std::ptr::null_mut(),verb.as_ptr(),target.as_ptr(),std::ptr::null(),std::ptr::null(),1)};
    if result as isize > 32 {Ok(())}else{Err(format!("Could not open external resource: {}",result as isize))}
}
pub fn available_memory_bytes()->Result<u64,String> {
    let mut status:MEMORYSTATUSEX=unsafe {std::mem::zeroed()};status.dwLength=std::mem::size_of_val(&status) as u32;
    if unsafe {GlobalMemoryStatusEx(&mut status)}==0 {return Err(error())}Ok(status.ullAvailPhys)
}
#[link(name="ntdll")]
unsafe extern "system" {fn RtlGetVersion(info:*mut OSVERSIONINFOW)->i32;fn NtResumeProcess(process:HANDLE)->i32;}
pub fn kernel_version()->Result<String,String> {
    let mut info:OSVERSIONINFOW=unsafe {std::mem::zeroed()};info.dwOSVersionInfoSize=std::mem::size_of_val(&info) as u32;
    if unsafe{RtlGetVersion(&mut info)}<0{return Err("Cannot detect Windows version".into())}
    Ok(format!("{}.{}.{}",info.dwMajorVersion,info.dwMinorVersion,info.dwBuildNumber))
}
pub fn move_without_replace(source:&Path,destination:&Path)->Result<(),String> {
    let source=wide(source)?;let destination=wide(destination)?;
    if unsafe {windows_sys::Win32::Storage::FileSystem::MoveFileExW(source.as_ptr(),destination.as_ptr(),0)}!=0{return Ok(())}
    let e=std::io::Error::last_os_error();
    if e.kind()==std::io::ErrorKind::AlreadyExists{return Err("A file with this name already exists in the destination; no files were replaced".into())}Err(e.to_string())
}
struct Handle(HANDLE);
impl Drop for Handle {fn drop(&mut self){unsafe{CloseHandle(self.0);}}}
pub fn protect_private_file(file:&std::fs::File)->Result<(),String> {
    unsafe {
        // Standard file handles do not request WRITE_DAC; reopen the same file
        // object instead of resolving a path that could change under us.
        use windows_sys::Win32::Storage::FileSystem::{ReOpenFile,WRITE_DAC,READ_CONTROL,FILE_SHARE_READ,FILE_SHARE_WRITE,FILE_SHARE_DELETE};
        let security_handle=ReOpenFile(file.as_raw_handle() as HANDLE,WRITE_DAC|READ_CONTROL,FILE_SHARE_READ|FILE_SHARE_WRITE|FILE_SHARE_DELETE,0);
        if security_handle==INVALID_HANDLE_VALUE{return Err(format!("Reopening credential file: {}",error()))}
        let security_handle=Handle(security_handle);
        let mut token=std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(),TOKEN_QUERY,&mut token)==0{return Err(error())}
        let token=Handle(token);let mut size=0;
        GetTokenInformation(token.0,TokenUser,std::ptr::null_mut(),0,&mut size);
        if size==0{return Err(error())}
        let mut buffer=vec![0u64;(size as usize+7)/8];
        if GetTokenInformation(token.0,TokenUser,buffer.as_mut_ptr().cast(),size,&mut size)==0{return Err(error())}
        let user=&*(buffer.as_ptr().cast::<TOKEN_USER>());
        let mut entry:EXPLICIT_ACCESS_W=std::mem::zeroed();
        entry.grfAccessPermissions=windows_sys::Win32::Storage::FileSystem::FILE_ALL_ACCESS;
        entry.grfAccessMode=SET_ACCESS;entry.grfInheritance=NO_INHERITANCE;
        entry.Trustee.TrusteeForm=TRUSTEE_IS_SID;entry.Trustee.TrusteeType=TRUSTEE_IS_USER;entry.Trustee.ptstrName=user.User.Sid.cast();
        let mut acl=std::ptr::null_mut();
        let status=SetEntriesInAclW(1,&entry,std::ptr::null(),&mut acl);
        if status!=ERROR_SUCCESS{return Err(format!("Setting credential ACL: {}",std::io::Error::from_raw_os_error(status as i32)))}
        let status=SetSecurityInfo(security_handle.0,SE_FILE_OBJECT,DACL_SECURITY_INFORMATION|PROTECTED_DACL_SECURITY_INFORMATION,std::ptr::null_mut(),std::ptr::null_mut(),acl,std::ptr::null());
        LocalFree(acl.cast());
        if status!=ERROR_SUCCESS{return Err(format!("Setting credential ACL: {}",std::io::Error::from_raw_os_error(status as i32)))}
    }Ok(())
}
/// Created suspended so installer processors cannot escape before job assignment.
pub struct InstallerChild {pub child:Child,_job:Handle}
impl std::ops::Deref for InstallerChild {type Target=Child;fn deref(&self)->&Child{&self.child}}
impl std::ops::DerefMut for InstallerChild {fn deref_mut(&mut self)->&mut Child{&mut self.child}}
pub fn spawn_installer(command:&mut Command)->Result<InstallerChild,String> {
    unsafe {
        let job=CreateJobObjectW(std::ptr::null(),std::ptr::null());if job.is_null(){return Err(error())}let job=Handle(job);
        let mut limits:JOBOBJECT_EXTENDED_LIMIT_INFORMATION=std::mem::zeroed();limits.BasicLimitInformation.LimitFlags=JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if SetInformationJobObject(job.0,JobObjectExtendedLimitInformation,(&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),std::mem::size_of_val(&limits) as u32)==0{return Err(error())}
        let mut child=command.creation_flags(CREATE_SUSPENDED|CREATE_NO_WINDOW).spawn().map_err(|e|e.to_string())?;
        let handle=child.as_raw_handle() as HANDLE;
        if AssignProcessToJobObject(job.0,handle)==0 {let e=error();let _=child.kill();let _=child.wait();return Err(e)}
        if NtResumeProcess(handle)<0 {let _=child.kill();let _=child.wait();return Err("Cannot resume installer process".into())}
        Ok(InstallerChild{child,_job:job})
    }
}
pub fn kill_installer(child:&mut InstallerChild) {
    unsafe {TerminateJobObject(child._job.0,1);}
    let _=child.child.kill();let _=child.child.wait();
}
pub fn steam_keyboard(show:bool)->Result<(),String> {
    open_external(if show {"steam://open/keyboard?XPosition=0&YPosition=0&Width=0&Height=0&Mode=0"}else{"steam://close/keyboard"})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn host_memory_paths_and_no_overwrite_move() {
        assert!(available_memory_bytes().unwrap()>0);
        assert!(!kernel_version().unwrap().is_empty());
        assert_eq!(crate::child_path(Path::new(r"\\?\C:\Games\Java")),r"C:\Games\Java");
        let root=std::env::temp_dir().join(format!("mcla-platform-{}",std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let source=root.join("source");let destination=root.join("destination");
        std::fs::write(&source,b"new").unwrap();std::fs::write(&destination,b"old").unwrap();
        assert!(move_without_replace(&source,&destination).is_err());
        assert_eq!(std::fs::read(&destination).unwrap(),b"old");
        std::fs::remove_file(&destination).unwrap();
        move_without_replace(&source,&destination).unwrap();
        assert_eq!(std::fs::read(&destination).unwrap(),b"new");
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn private_storage_can_set_acl_on_standard_file_handle() {
        let path=std::env::temp_dir().join(format!("mcla-private-{}",std::process::id()));
        let file=std::fs::File::create(&path).unwrap();
        protect_private_file(&file).unwrap();
        drop(file);std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn installer_process_can_be_started_and_cancelled() {
        let mut command=Command::new("cmd.exe");command.args(["/C","ping -n 60 127.0.0.1 > nul"]);
        let mut child=spawn_installer(&mut command).unwrap();
        assert!(child.try_wait().unwrap().is_none());
        kill_installer(&mut child);
        assert!(child.try_wait().unwrap().is_some());
    }
}
