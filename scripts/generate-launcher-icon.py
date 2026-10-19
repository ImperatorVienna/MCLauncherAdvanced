"""Draw a shared orthographic isometric grid; texture borrowed from existing grass icon."""
from pathlib import Path
import math, re
root=Path(__file__).resolve().parents[1]
source=(root/'apps/desktop/src/assets/installation-icons/grass.svg').read_text()
images=re.findall(r'href="([^"]+)"',source)
a=math.sqrt(3)/2
L=360
cx,cy=450,60
left=(cx-a*L,cy+L/2)
center=(cx,cy+L)
s=['<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -25 900 900" role="img" aria-labelledby="title desc">', '<title id="title">MCLA grass block logo</title><desc id="desc">Orthographic isometric cube. MC on left face, LA on right face. All faces and glyphs share axes at plus or minus thirty degrees.</desc>', '<defs>'+source.split('<defs>')[1].split('</defs>')[0]+'</defs>']
def texture(matrix,href,filter=''):
 s.append(f'<image width="16" height="16" transform="matrix({matrix})" href="{href}" style="image-rendering:pixelated" {filter}/>')
k=L/16
texture(f'{a*k} {k/2} {-a*k} {k/2} {cx} {cy}',images[0],'filter="url(#grass)"')
for x,y,sign,shade in [(*left,1,.13),(*center,-1,.3)]:
 mat=f'{a*k} {sign*k/2} 0 {k} {x} {y}'
 texture(mat,images[1]);texture(mat,images[2],'filter="url(#grass)"')
 s.append(f'<path transform="matrix({a} {sign*.5} 0 1 {x} {y})" d="M0 0H{L}V{L}H0Z" fill="black" opacity="{shade}"/>')
letters={
 'M':['1100011','1110111','1111111','1101011','1100011','1100011','1100011','1100011','1100011'],
 'C':['0111111','1111111','1100000','1100000','1100000','1100000','1100000','1111111','0111111'],
 'L':['1100000','1100000','1100000','1100000','1100000','1100000','1100000','1111111','1111111'],
 'A':['0111110','1111111','1100011','1100011','1111111','1111111','1100011','1100011','1100011'],
}
def glyph(letter,x0,y0,ox,oy,sign):
 cell=18
 cells={(x,y) for y,row in enumerate(letters[letter]) for x,c in enumerate(row) if c=='1'}
 def project(x,y):return (ox+a*(x0+x*cell),oy+sign*.5*(x0+x*cell)+y0+y*cell)
 # Outward normal is the other horizontal cube axis; same orthographic projection.
 dx,dy=(-sign*a*18,9)
 def points(ps):return ' '.join(f'{x:.6f},{y:.6f}' for x,y in ps)
 edges=[]
 for x,y in sorted(cells):
  for neighbor,p,q in [((x,y-1),(x,y),(x+1,y)),((x+1,y),(x+1,y),(x+1,y+1)),((x,y+1),(x+1,y+1),(x,y+1)),((x-1,y),(x,y+1),(x,y))]:
   if neighbor not in cells:edges.append((project(*p),project(*q)))
 # rear outline, side extrusion, and front faces are all projected from the identical grid.
 for p,q in edges:
  s.append(f'<path d="M{p[0]} {p[1]}L{q[0]} {q[1]}" stroke="#171a1b" stroke-width="9"/>')
 for p,q in edges:
  s.append(f'<polygon points="{points([p,q,(q[0]+dx,q[1]+dy),(p[0]+dx,p[1]+dy)])}" fill="#424649" stroke="#24282b" stroke-width="1"/>')
 for x,y in sorted(cells):
  ps=[project(x,y),project(x+1,y),project(x+1,y+1),project(x,y+1)]
  ps=[(px+dx,py+dy) for px,py in ps]
  color=['#a7a9aa','#b7b9ba','#96999b'][(x*3+y*7)%3]
  s.append(f'<polygon points="{points(ps)}" fill="{color}" stroke="{color}" stroke-width=".35"/>')
 for p,q in edges:
  s.append(f'<path d="M{p[0]+dx} {p[1]+dy}L{q[0]+dx} {q[1]+dy}" fill="none" stroke="#202527" stroke-width="3" stroke-linejoin="miter"/>')
for pair,(ox,oy),sign in [('MC',left,1),('LA',center,-1)]:
 for letter,x in zip(pair,[43,197]):glyph(letter,x,135,ox,oy,sign)
s.append('</svg>')
out=root/'apps/desktop/src-tauri/icons/icon.svg'
out.write_text('\n'.join(s))
assert abs(math.degrees(math.atan2(.5,a))-30)<1e-12
print(out)

# Derived host/browser assets. Requires librsvg (rsvg-convert) and Pillow.
import subprocess
from PIL import Image
subprocess.run(['rsvg-convert','-w','512','-h','512',str(out),'-o',str(out.with_suffix('.png'))],check=True)
with Image.open(out.with_suffix('.png')) as source_image:
 image=source_image.convert('RGBA')
 image.save(out.with_suffix('.ico'),sizes=[(16,16),(24,24),(32,32),(48,48),(64,64),(128,128),(256,256)])
 public=root/'apps/desktop/public'
 public.mkdir(exist_ok=True)
 image.resize((64,64),Image.Resampling.LANCZOS).save(public/'favicon.png')
