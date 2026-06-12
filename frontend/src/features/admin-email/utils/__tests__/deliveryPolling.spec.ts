import { describe, expect, it, vi } from 'vitest'
import type { AsyncTaskItem, AsyncTaskStatus } from '@/api/async-tasks'
import { waitForEmailDeliveryResult } from '../deliveryPolling'

function task(status: AsyncTaskStatus, overrides: Partial<AsyncTaskItem> = {}): AsyncTaskItem {
  return {
    id: 'delivery-1',
    status,
    progress_percent: status === 'succeeded' ? 100 : 10,
    progress_message: null,
    error_message: null,
    created_at: '2026-06-11T00:00:00Z',
    ...overrides,
  }
}

describe('waitForEmailDeliveryResult', () => {
  it('waits until the email delivery task succeeds', async () => {
    let currentTime = 0
    const fetchTask = vi
      .fn()
      .mockResolvedValueOnce(task('queued'))
      .mockResolvedValueOnce(task('succeeded', { progress_message: '邮件已发送' }))
    const sleep = vi.fn(async (ms: number) => {
      currentTime += ms
    })

    const result = await waitForEmailDeliveryResult('delivery-1', {
      fetchTask,
      sleep,
      now: () => currentTime,
      intervalMs: 2000,
      timeoutMs: 10_000,
    })

    expect(result).toEqual({
      status: 'succeeded',
      task: task('succeeded', { progress_message: '邮件已发送' }),
      message: '邮件已发送',
    })
    expect(fetchTask).toHaveBeenCalledTimes(2)
    expect(sleep).toHaveBeenCalledWith(2000)
  })

  it('returns the task error when the email delivery task fails', async () => {
    const fetchTask = vi.fn().mockResolvedValue(
      task('failed', {
        progress_message: '邮件发送失败',
        error_message: '用户名或密码错误',
      }),
    )

    const result = await waitForEmailDeliveryResult('delivery-1', {
      fetchTask,
      sleep: vi.fn(),
    })

    expect(result.status).toBe('failed')
    expect(result.message).toBe('用户名或密码错误')
  })

  it('returns timeout when the task keeps running past the wait window', async () => {
    const fetchTask = vi.fn().mockResolvedValue(task('running'))

    const result = await waitForEmailDeliveryResult('delivery-1', {
      fetchTask,
      sleep: vi.fn(),
      now: () => 1000,
      timeoutMs: 0,
    })

    expect(result.status).toBe('timeout')
    expect(result.message).toBe('测试邮件仍在发送，请在最近发送记录查看结果')
    expect(fetchTask).toHaveBeenCalledTimes(1)
  })
})
