import { asyncTasksApi, type AsyncTaskItem, type AsyncTaskStatus } from '@/api/async-tasks'

const DEFAULT_TIMEOUT_MS = 90_000
const DEFAULT_INTERVAL_MS = 2_000

export type EmailDeliveryWaitStatus = 'succeeded' | 'failed' | 'timeout'

export interface EmailDeliveryWaitResult {
  status: EmailDeliveryWaitStatus
  task?: AsyncTaskItem
  message: string
}

export interface EmailDeliveryWaitOptions {
  timeoutMs?: number
  intervalMs?: number
  fetchTask?: (taskId: string) => Promise<AsyncTaskItem>
  sleep?: (ms: number) => Promise<void>
  now?: () => number
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => window.setTimeout(resolve, ms))
}

function isSuccessStatus(status: AsyncTaskStatus): boolean {
  return status === 'succeeded' || status === 'completed'
}

function isFailureStatus(status: AsyncTaskStatus): boolean {
  return status === 'failed' || status === 'cancelled' || status === 'skipped'
}

function taskFailureMessage(task: AsyncTaskItem): string {
  return task.error_message || task.progress_message || '测试邮件发送失败'
}

export async function waitForEmailDeliveryResult(
  deliveryId: string,
  options: EmailDeliveryWaitOptions = {},
): Promise<EmailDeliveryWaitResult> {
  const timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS
  const intervalMs = options.intervalMs ?? DEFAULT_INTERVAL_MS
  const fetchTask = options.fetchTask ?? asyncTasksApi.getDetail
  const wait = options.sleep ?? sleep
  const now = options.now ?? Date.now
  const deadline = now() + timeoutMs
  let latestTask: AsyncTaskItem | undefined

  while (true) {
    latestTask = await fetchTask(deliveryId)

    if (isSuccessStatus(latestTask.status)) {
      return {
        status: 'succeeded',
        task: latestTask,
        message: latestTask.progress_message || '测试邮件已发送',
      }
    }

    if (isFailureStatus(latestTask.status)) {
      return {
        status: 'failed',
        task: latestTask,
        message: taskFailureMessage(latestTask),
      }
    }

    const remainingMs = deadline - now()
    if (remainingMs <= 0) {
      return {
        status: 'timeout',
        task: latestTask,
        message: '测试邮件仍在发送，请在最近发送记录查看结果',
      }
    }

    await wait(Math.min(intervalMs, remainingMs))
  }
}
