export default function CopyIcon({
  size = 32,
  color = "#000000",
  className = "",
}: {
  size?: number;
  color?: string;
  className?: string;
}) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 32 32"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
      style={{ width: size, height: size, flexShrink: 0 }}
    >
      <path
        d="M10 4H26V6H28V22H26V24H10V22H8V6H10V4ZM24 20V8H12V20H24Z"
        fill={color}
        fillRule="evenodd"
        clipRule="evenodd"
      />
      <path
        d="M4 10H20V12H22V28H20V30H4V28H2V12H4V10ZM18 26V14H6V26H18Z"
        fill={color}
        fillRule="evenodd"
        clipRule="evenodd"
      />
    </svg>
  );
}