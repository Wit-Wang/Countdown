import type { DateTime } from '../types/interface';

export function dateTimeToTs(dt: DateTime): number {
  if (!dt) return NaN;
  const y = dt.year ?? 0;
  const M = (dt.month ?? 1) - 1;
  const d = dt.day ?? 1;
  const h = dt.hour ?? 0;
  const m = dt.minute ?? 0;
  const s = dt.second ?? 0;
  return new Date(y, M, d, h, m, s).getTime();
}

export function tsToDateTime(ts: number): DateTime {
  const date = new Date(ts);
  return {
    year: date.getFullYear(),
    month: date.getMonth() + 1,
    day: date.getDate(),
    hour: date.getHours(),
    minute: date.getMinutes(),
    second: date.getSeconds(),
  };
}

export function formatDateTime(dt: DateTime): string {
  const ts = dateTimeToTs(dt);
  const d = new Date(ts);
  const y = d.getFullYear();
  const M = String(d.getMonth() + 1).padStart(2, '0');
  const D = String(d.getDate()).padStart(2, '0');
  const h = String(d.getHours()).padStart(2, '0');
  const m = String(d.getMinutes()).padStart(2, '0');
  return `${y}-${M}-${D} ${h}:${m}`;
}

export function dateFromParts(
  year: number,
  month: number,
  day: number,
  hour = 0,
  minute = 0,
  second = 0
): DateTime {
  return { year, month, day, hour, minute, second };
}

export function addIntervalToDateTime(base: DateTime, interval: DateTime): DateTime {
  const ts = dateTimeToTs(base);
  const d = new Date(ts);
  d.setFullYear(d.getFullYear() + (interval.year ?? 0));
  d.setMonth(d.getMonth() + (interval.month ?? 0));
  d.setDate(d.getDate() + (interval.day ?? 0));
  d.setHours(d.getHours() + (interval.hour ?? 0));
  d.setMinutes(d.getMinutes() + (interval.minute ?? 0));
  d.setSeconds(d.getSeconds() + (interval.second ?? 0));
  return tsToDateTime(d.getTime());
}
