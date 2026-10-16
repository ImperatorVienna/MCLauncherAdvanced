import { iconImages, type IconId } from "../state/instanceIcons";
export function InstanceIcon({ icon = "grass" }: { icon?: IconId }) {
  return <img className="grass-icon installation-icon" src={iconImages[icon] ?? iconImages.grass} alt="" aria-hidden="true" draggable={false} />;
}
