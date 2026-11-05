/** A key combination drawn as one key cap per key (see `displayKeys`). */
export function Keys({ keys }: { keys: string[] }) {
  return (
    <span className="keys">
      {keys.map((k, i) => (
        <kbd key={i}>{k}</kbd>
      ))}
    </span>
  );
}
