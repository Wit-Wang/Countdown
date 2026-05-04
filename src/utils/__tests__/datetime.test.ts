import { describe, it, expect } from 'vitest';
import {
  dateTimeToTs,
  tsToDateTime,
  formatDateTime,
  dateFromParts,
  addIntervalToDateTime,
} from '../datetime';
import type { DateTime } from '../../types/interface';

describe('dateTimeToTs', () => {
  it('converts a valid DateTime to timestamp', () => {
    const dt: DateTime = { year: 2026, month: 5, day: 4, hour: 14, minute: 30, second: 0 };
    const ts = dateTimeToTs(dt);
    const d = new Date(ts);
    expect(d.getFullYear()).toBe(2026);
    expect(d.getMonth()).toBe(4); // 0-indexed
    expect(d.getDate()).toBe(4);
    expect(d.getHours()).toBe(14);
    expect(d.getMinutes()).toBe(30);
  });

  it('handles January (month=1)', () => {
    const dt: DateTime = { year: 2026, month: 1, day: 15, hour: 0, minute: 0, second: 0 };
    const ts = dateTimeToTs(dt);
    expect(new Date(ts).getMonth()).toBe(0);
  });

  it('handles December (month=12)', () => {
    const dt: DateTime = { year: 2026, month: 12, day: 25, hour: 0, minute: 0, second: 0 };
    const ts = dateTimeToTs(dt);
    expect(new Date(ts).getMonth()).toBe(11);
  });

  it('handles leap year Feb 29', () => {
    const dt: DateTime = { year: 2024, month: 2, day: 29, hour: 0, minute: 0, second: 0 };
    const ts = dateTimeToTs(dt);
    expect(new Date(ts).getMonth()).toBe(1);
    expect(new Date(ts).getDate()).toBe(29);
  });

  it('returns NaN for null', () => {
    expect(dateTimeToTs(null as unknown as DateTime)).toBeNaN();
  });

  it('returns NaN for undefined', () => {
    expect(dateTimeToTs(undefined as unknown as DateTime)).toBeNaN();
  });

  it('defaults missing fields to zero', () => {
    const dt = { year: 2026 } as DateTime;
    const ts = dateTimeToTs(dt);
    const d = new Date(ts);
    expect(d.getMonth()).toBe(0);
    expect(d.getDate()).toBe(1);
    expect(d.getHours()).toBe(0);
    expect(d.getMinutes()).toBe(0);
    expect(d.getSeconds()).toBe(0);
  });
});

describe('tsToDateTime', () => {
  it('converts a timestamp back to DateTime', () => {
    const ts = new Date(2026, 4, 4, 14, 30, 0).getTime();
    const dt = tsToDateTime(ts);
    expect(dt.year).toBe(2026);
    expect(dt.month).toBe(5);
    expect(dt.day).toBe(4);
    expect(dt.hour).toBe(14);
    expect(dt.minute).toBe(30);
    expect(dt.second).toBe(0);
  });

  it('round-trips with dateTimeToTs', () => {
    const original: DateTime = { year: 2026, month: 12, day: 25, hour: 8, minute: 15, second: 30 };
    const ts = dateTimeToTs(original);
    const back = tsToDateTime(ts);
    expect(back).toEqual(original);
  });
});

describe('formatDateTime', () => {
  it('formats as YYYY-MM-DD HH:mm', () => {
    const dt: DateTime = { year: 2026, month: 5, day: 4, hour: 14, minute: 30, second: 0 };
    expect(formatDateTime(dt)).toBe('2026-05-04 14:30');
  });

  it('pads single-digit month and day', () => {
    const dt: DateTime = { year: 2026, month: 1, day: 5, hour: 9, minute: 5, second: 0 };
    expect(formatDateTime(dt)).toBe('2026-01-05 09:05');
  });
});

describe('dateFromParts', () => {
  it('creates DateTime from year, month, day', () => {
    expect(dateFromParts(2026, 5, 4)).toEqual({
      year: 2026, month: 5, day: 4, hour: 0, minute: 0, second: 0,
    });
  });

  it('accepts hour, minute, second', () => {
    expect(dateFromParts(2026, 5, 4, 14, 30, 15)).toEqual({
      year: 2026, month: 5, day: 4, hour: 14, minute: 30, second: 15,
    });
  });
});

describe('addIntervalToDateTime', () => {
  it('adds days', () => {
    const base: DateTime = { year: 2026, month: 5, day: 4, hour: 0, minute: 0, second: 0 };
    const interval: DateTime = { year: 0, month: 0, day: 1, hour: 0, minute: 0, second: 0 };
    const result = addIntervalToDateTime(base, interval);
    expect(result.year).toBe(2026);
    expect(result.month).toBe(5);
    expect(result.day).toBe(5);
  });

  it('adds months crossing year boundary', () => {
    const base: DateTime = { year: 2026, month: 11, day: 1, hour: 0, minute: 0, second: 0 };
    const interval: DateTime = { year: 0, month: 2, day: 0, hour: 0, minute: 0, second: 0 };
    const result = addIntervalToDateTime(base, interval);
    expect(result.year).toBe(2027);
    expect(result.month).toBe(1);
  });

  it('adds multiple fields simultaneously', () => {
    const base: DateTime = { year: 2026, month: 1, day: 1, hour: 10, minute: 0, second: 0 };
    const interval: DateTime = { year: 1, month: 1, day: 5, hour: 2, minute: 30, second: 0 };
    const result = addIntervalToDateTime(base, interval);
    expect(result.year).toBe(2027);
    expect(result.month).toBe(2);
    expect(result.day).toBe(6);
    expect(result.hour).toBe(12);
    expect(result.minute).toBe(30);
  });

  it('subtracts with negative interval', () => {
    const base: DateTime = { year: 2026, month: 5, day: 10, hour: 0, minute: 0, second: 0 };
    const interval: DateTime = { year: 0, month: 0, day: -3, hour: 0, minute: 0, second: 0 };
    const result = addIntervalToDateTime(base, interval);
    expect(result.year).toBe(2026);
    expect(result.month).toBe(5);
    expect(result.day).toBe(7);
  });

  it('preserves time when only date changes', () => {
    const base: DateTime = { year: 2026, month: 5, day: 4, hour: 14, minute: 30, second: 15 };
    const interval: DateTime = { year: 0, month: 1, day: 0, hour: 0, minute: 0, second: 0 };
    const result = addIntervalToDateTime(base, interval);
    expect(result.hour).toBe(14);
    expect(result.minute).toBe(30);
    expect(result.second).toBe(15);
  });
});
