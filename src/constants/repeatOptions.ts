import type { DateTime } from '../types/interface';

export const REPEAT_OPTIONS: Record<string, { label: string; interval: DateTime | null }> = {
  none: { label: '无', interval: null },
  day: { label: '一天', interval: { year: 0, month: 0, day: 1, hour: 0, minute: 0, second: 0 } },
  every2days: {
    label: '每隔一天',
    interval: { year: 0, month: 0, day: 2, hour: 0, minute: 0, second: 0 },
  },
  week: { label: '一周', interval: { year: 0, month: 0, day: 7, hour: 0, minute: 0, second: 0 } },
  every2weeks: {
    label: '每隔一周',
    interval: { year: 0, month: 0, day: 14, hour: 0, minute: 0, second: 0 },
  },
};

export const REPEAT_KEYS = Object.keys(REPEAT_OPTIONS) as Array<keyof typeof REPEAT_OPTIONS>;

export default REPEAT_OPTIONS;
