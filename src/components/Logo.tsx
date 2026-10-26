export function Logo({ size = 22 }: { size?: number }) {
  return (
    <svg className="logo" viewBox="0 0 24 24" width={size} height={size} fill="none" strokeWidth="2.6" strokeLinecap="round">
      <circle cx="12" cy="12" r="9" stroke="var(--track)" />
      <path d="M12 3a9 9 0 1 1-8.2 5.3" stroke="var(--accent)" />
      <circle cx="12" cy="12" r="3.2" fill="var(--warn)" stroke="none" />
    </svg>
  );
}
