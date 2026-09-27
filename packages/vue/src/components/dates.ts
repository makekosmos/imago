// Локальные даты для Calendar / DateTimePicker.
//
// `new Date(y, m, d)` трактует year 0..99 как 1900+y, поэтому год ставится
// через setFullYear. Важно: поля задаются ОДНИМ вызовом
// `setFullYear(year, month, day)` — переполнение дня тогда вычисляется в
// ЦЕЛЕВОМ году. Сборка через `new Date(0, m, d)` + `setFullYear(y)` считала
// overflow в 1900-м (невисокосном) и сдвигала 29 февраля високосного года
// на 1 марта.

export function pad(n: number): string {
  return String(n).padStart(2, "0");
}

export function formatIsoYear(year: number): string {
  if (year < 0) return `-${String(Math.abs(year)).padStart(6, "0")}`;
  return String(year).padStart(4, "0");
}

/** Локальная полночь заданной даты (день может переполняться, как в Date). */
export function localDate(year: number, month: number, day: number): Date {
  const d = new Date(0);
  d.setFullYear(year, month, day);
  d.setHours(0, 0, 0, 0);
  return d;
}

export interface LocalDateTimeParts {
  year: number;
  month: number; // 0..11
  day: number;
  hour: number;
  minute: number;
}

/** Локальная дата-время; day/month переполняются в целевом году. */
export function localDateTime(parts: LocalDateTimeParts): Date {
  const d = new Date(0);
  d.setFullYear(parts.year, parts.month, parts.day);
  d.setHours(parts.hour, parts.minute, 0, 0);
  return d;
}
