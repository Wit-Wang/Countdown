import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { showToast, toast } from '../toast';

beforeEach(() => {
  vi.useFakeTimers();
  toast.value = { show: false, msg: '', type: 'info' };
});

afterEach(() => {
  vi.useRealTimers();
});

describe('showToast', () => {
  it('sets toast show to true', () => {
    showToast('hello');
    expect(toast.value.show).toBe(true);
  });

  it('sets the message', () => {
    showToast('hello');
    expect(toast.value.msg).toBe('hello');
  });

  it('defaults type to info', () => {
    showToast('hello');
    expect(toast.value.type).toBe('info');
  });

  it('accepts custom type', () => {
    showToast('ok', 'success');
    expect(toast.value.type).toBe('success');
    showToast('err', 'error');
    expect(toast.value.type).toBe('error');
  });

  it('hides toast after 2500ms', () => {
    showToast('hello');
    expect(toast.value.show).toBe(true);
    vi.advanceTimersByTime(2500);
    expect(toast.value.show).toBe(false);
  });

  it('clears previous timer when called again', () => {
    showToast('first');
    vi.advanceTimersByTime(1000);
    showToast('second');
    vi.advanceTimersByTime(2500);
    expect(toast.value.msg).toBe('second');
    expect(toast.value.show).toBe(false);
  });
});
