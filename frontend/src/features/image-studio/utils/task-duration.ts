export function formatTaskDuration(
  startedAt?: number,
  finishedAt?: number,
  now = Date.now(),
) {
  if (!startedAt || !Number.isFinite(startedAt)) return ''
  const end = finishedAt && Number.isFinite(finishedAt) ? finishedAt : now
  const totalSeconds = Math.max(0, end - startedAt) / 1000

  if (totalSeconds < 60) return `${totalSeconds.toFixed(1)}s`

  const totalMinutes = Math.floor(totalSeconds / 60)
  const seconds = totalSeconds - totalMinutes * 60
  if (totalMinutes < 60) return `${totalMinutes}m ${seconds.toFixed(1)}s`

  const hours = Math.floor(totalMinutes / 60)
  const minutes = totalMinutes - hours * 60
  return `${hours}h ${minutes}m ${seconds.toFixed(1)}s`
}
