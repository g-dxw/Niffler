import { describe, expect, it } from 'vitest'
import { formatTaskDuration } from '../utils/task-duration'

describe('image task duration', () => {
  it('uses the current time while a task is running', () => {
    expect(formatTaskDuration(1_000, undefined, 3_340)).toBe('2.3s')
  })

  it('freezes at the recorded finish time and formats long durations', () => {
    expect(formatTaskDuration(1_000, 66_500, 999_999)).toBe('1m 5.5s')
    expect(formatTaskDuration(1_000, 3_667_500)).toBe('1h 1m 6.5s')
  })

  it('never displays a negative duration', () => {
    expect(formatTaskDuration(5_000, 4_000)).toBe('0.0s')
    expect(formatTaskDuration()).toBe('')
  })
})
