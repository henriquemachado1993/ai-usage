import { getCurrentWindow } from "@tauri-apps/api/window";

type Direction = "East" | "West" | "South" | "SouthEast" | "SouthWest";

const HANDLES: { dir: Direction; className: string }[] = [
  { dir: "East", className: "rh-e" },
  { dir: "West", className: "rh-w" },
  { dir: "South", className: "rh-s" },
  { dir: "SouthEast", className: "rh-se" },
  { dir: "SouthWest", className: "rh-sw" },
];

export function ResizeHandles() {
  return (
    <>
      {HANDLES.map(({ dir, className }) => (
        <div
          key={dir}
          className={`rh ${className}`}
          onMouseDown={(e) => {
            if (e.button === 0) getCurrentWindow().startResizeDragging(dir);
          }}
        />
      ))}
    </>
  );
}
