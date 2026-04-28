export interface Todo {
  id: number;
  text: string;
  deadline: DateTime;
  repeat?: DateTime | null;
  completed: boolean;
}

export interface DateTime {
  year: number;
  month: number;
  day: number;
  hour: number;
  minute: number;
  second: number;
}

export const URGENCY = [
  [-Infinity, 0, '已到期', '#e53935'],
  [0, 24 * 60 * 60 * 1000, '紧急', '#fb8c00'],
  [24 * 60 * 60 * 1000, 3 * 24 * 60 * 60 * 1000, '即将', '#1e88e5'],
  [3 * 24 * 60 * 60 * 1000, Infinity, '待办', '#43a047'],
];
