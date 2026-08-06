import { useClock } from "../hooks/useClock";

interface ClockProps {
  message?: string;
}

export function Clock({ message }: ClockProps) {
  const now = useClock();
  const time = new Intl.DateTimeFormat(undefined, {
    hour: "numeric",
    minute: "2-digit",
  }).format(now);
  const date = new Intl.DateTimeFormat(undefined, {
    weekday: "long",
    month: "long",
    day: "numeric",
  }).format(now);

  return (
    <div className="clock" aria-live="off">
      <p className="clock__time">{time}</p>
      <p className="clock__date">{date}</p>
      {message && <p className="clock__message">{message}</p>}
    </div>
  );
}
