/**
 * 解析 API 错误响应，提取友好的错误信息
 */

import { isApiError } from '@/types/api-error'
import { i18n } from '@/i18n'

/**
 * Pydantic 验证错误项
 */
interface ValidationError {
  loc: (string | number)[]
  msg: string
  type: string
  ctx?: Record<string, unknown>
}

// Store and utility callers still pass legacy Chinese fallback messages. Keep
// those fallbacks localized centrally so exceptional paths also respect the
// selected language without changing every caller at once.
const legacyFallbackTranslations: Record<string, string> = {
  '获取模块状态失败': 'Failed to load module status',
  '设置模块状态失败': 'Failed to update module status',
  '获取代理节点列表失败': 'Failed to load proxy nodes',
  '创建手动代理节点失败': 'Failed to create proxy node',
  '生成代理节点安装命令失败': 'Failed to generate the proxy-node install command',
  '删除代理节点失败': 'Failed to delete proxy node',
  '获取用户列表失败': 'Failed to load users',
  '创建用户失败': 'Failed to create user',
  '更新用户失败': 'Failed to update user',
  '删除用户失败': 'Failed to delete user',
  '解析用户选择失败': 'Failed to resolve the user selection',
  '批量操作用户失败': 'Failed to run the batch user action',
  '获取用户分组失败': 'Failed to load user groups',
  '创建用户分组失败': 'Failed to create user group',
  '更新用户分组失败': 'Failed to update user group',
  '删除用户分组失败': 'Failed to delete user group',
  '替换 Key 分组并删除失败': 'Failed to replace the Key group and delete it',
  '获取分组成员失败': 'Failed to load group members',
  '更新分组成员失败': 'Failed to update group members',
  '设置默认用户组失败': 'Failed to set the default user group',
  '获取 API Keys 失败': 'Failed to load API keys',
  '创建 API Key 失败': 'Failed to create API key',
  '更新 API Key 失败': 'Failed to update API key',
  '删除 API Key 失败': 'Failed to delete API key',
  '获取完整 API Key 失败': 'Failed to load the full API key',
  '获取用户设备会话失败': 'Failed to load user sessions',
  '获取用户套餐失败': 'Failed to load user plans',
  '发放用户套餐失败': 'Failed to grant user plan',
  '强制下线设备失败': 'Failed to sign out the device',
  '强制下线全部设备失败': 'Failed to sign out all devices',
  '操作失败': 'Operation failed',
}

function localizeLegacyFallback(message: string): string {
  if (i18n.global.locale.value !== 'en-US') return message
  return legacyFallbackTranslations[message] || message
}

/**
 * 字段名称映射（中文化）
 */
const fieldNameMap: Record<string, string> = {
  'api_key': i18n.global.t('errorUi.apiKey'),
  'priority': i18n.global.t('errorUi.priority'),
  'rpm_limit': i18n.global.t('errorUi.rpmLimit'),
  'rate_limit': i18n.global.t('errorUi.rateLimit'),
  'daily_limit': i18n.global.t('errorUi.dailyLimit'),
  'monthly_limit': i18n.global.t('errorUi.monthlyLimit'),
  'allowed_models': i18n.global.t('errorUi.allowedModels'),
  'note': i18n.global.t('errorUi.note'),
  'is_active': i18n.global.t('errorUi.active'),
  'endpoint_id': 'Endpoint ID',
  'base_url': i18n.global.t('errorUi.baseUrl'),
  'timeout': i18n.global.t('errorUi.timeout'),
  'max_retries': i18n.global.t('errorUi.maxRetries'),
  'weight': i18n.global.t('errorUi.weight'),
  'email': i18n.global.t('errorUi.email'),
  'username': i18n.global.t('errorUi.username'),
  'password': i18n.global.t('errorUi.password'),
  'name': i18n.global.t('errorUi.name'),
  'display_name': i18n.global.t('errorUi.displayName'),
  'description': i18n.global.t('errorUi.description'),
  'website': i18n.global.t('errorUi.website'),
  'provider_priority': i18n.global.t('errorUi.providerPriority'),
  'billing_type': i18n.global.t('errorUi.billingType'),
  'monthly_quota_usd': i18n.global.t('errorUi.monthlyQuota'),
  'quota_reset_day': i18n.global.t('errorUi.quotaResetDay'),
  'quota_expires_at': i18n.global.t('errorUi.quotaExpiresAt'),
  'cache_ttl_minutes': i18n.global.t('errorUi.cacheTtl'),
  'max_probe_interval_minutes': i18n.global.t('errorUi.maxProbeInterval'),
}

/**
 * 错误类型映射（中文化）
 */
const errorTypeMap: Record<string, (error: ValidationError) => string> = {
  'string_too_short': (error) => {
    const minLength = error.ctx?.min_length || 3
    return i18n.global.t('errorParserUi.lengthMin', { count: minLength })
  },
  'string_too_long': (error) => {
    const maxLength = error.ctx?.max_length
    return i18n.global.t('errorParserUi.lengthMax', { count: maxLength })
  },
  'value_error.missing': () => i18n.global.t('errorParserUi.required'),
  'missing': () => i18n.global.t('errorParserUi.required'),
  'type_error.none.not_allowed': () => i18n.global.t('errorParserUi.notNull'),
  'value_error': (error) => error.msg,
  'type_error.integer': () => i18n.global.t('errorParserUi.integer'),
  'type_error.float': () => i18n.global.t('errorParserUi.number'),
  'value_error.number.not_ge': (error) => {
    const limit = error.ctx?.limit_value
    return limit !== undefined ? i18n.global.t('errorParserUi.notLessThan', { value: limit }) : i18n.global.t('errorParserUi.tooSmall')
  },
  'value_error.number.not_le': (error) => {
    const limit = error.ctx?.limit_value
    return limit !== undefined ? i18n.global.t('errorParserUi.notGreaterThan', { value: limit }) : i18n.global.t('errorParserUi.tooLarge')
  },
  'value_error.number.not_gt': (error) => {
    const limit = error.ctx?.limit_value
    return limit !== undefined ? i18n.global.t('errorParserUi.greaterThan', { value: limit }) : i18n.global.t('errorParserUi.tooSmall')
  },
  'value_error.number.not_lt': (error) => {
    const limit = error.ctx?.limit_value
    return limit !== undefined ? i18n.global.t('errorParserUi.lessThan', { value: limit }) : i18n.global.t('errorParserUi.tooLarge')
  },
  'less_than_equal': (error) => {
    const limit = error.ctx?.le
    return limit !== undefined ? i18n.global.t('errorParserUi.notGreaterThan', { value: limit }) : i18n.global.t('errorParserUi.tooLarge')
  },
  'greater_than_equal': (error) => {
    const limit = error.ctx?.ge
    return limit !== undefined ? i18n.global.t('errorParserUi.notLessThan', { value: limit }) : i18n.global.t('errorParserUi.tooSmall')
  },
  'less_than': (error) => {
    const limit = error.ctx?.lt
    return limit !== undefined ? i18n.global.t('errorParserUi.lessThan', { value: limit }) : i18n.global.t('errorParserUi.tooLarge')
  },
  'greater_than': (error) => {
    const limit = error.ctx?.gt
    return limit !== undefined ? i18n.global.t('errorParserUi.greaterThan', { value: limit }) : i18n.global.t('errorParserUi.tooSmall')
  },
  'value_error.email': () => i18n.global.t('errorParserUi.emailInvalid'),
  'value_error.url': () => i18n.global.t('errorParserUi.urlInvalid'),
  'type_error.bool': () => i18n.global.t('errorParserUi.bool'),
  'type_error.list': () => i18n.global.t('errorParserUi.array'),
  'type_error.dict': () => i18n.global.t('errorParserUi.object'),
}

/**
 * 获取字段的中文名称
 */
function getFieldName(loc: (string | number)[]): string {
  if (!loc || loc.length === 0) return i18n.global.t('errorParserUi.field')

  const fieldPath = loc.filter(item => item !== 'body').join('.')
  const fieldKey = String(loc[loc.length - 1])

  return fieldNameMap[fieldKey] || fieldPath || i18n.global.t('errorParserUi.field')
}

/**
 * 格式化单个验证错误
 */
function formatValidationError(error: ValidationError): string {
  const fieldName = getFieldName(error.loc)
  const errorFormatter = errorTypeMap[error.type]

  if (errorFormatter) {
    const errorMsg = errorFormatter(error)
    return `${fieldName}: ${errorMsg}`
  }

  // 默认格式
  return `${fieldName}: ${error.msg}`
}

function normalizeKnownApiErrorMessage(message: string): string {
  const text = message.trim()
  if (!text) return text

  const lowered = text.toLowerCase()
  if (
    lowered.includes('refresh_token_reused')
    || lowered.includes('already been used to generate a new access token')
  ) {
    return i18n.global.t('errorParserFeedback.refreshTokenReused')
  }

  if (
    lowered.includes('refresh_token_expired')
    || lowered.includes('could not validate your refresh token')
  ) {
    return i18n.global.t('errorParserFeedback.refreshTokenInvalid')
  }

  if (
    lowered.includes('token refresh 失败:')
    || lowered.includes('token refresh failed:')
  ) {
    return text
      .replace(/^token refresh 失败:\s*/i, `${i18n.global.t('errorParserFeedback.refreshTokenPrefix')}: `)
      .replace(/^token refresh failed:\s*/i, `${i18n.global.t('errorParserFeedback.refreshTokenPrefix')}: `)
  }

  return text
}

/**
 * 解析 API 错误响应
 * @param err 错误对象
 * @param defaultMessage 默认错误信息
 * @returns 格式化的错误信息
 */
export function parseApiError(err: unknown, defaultMessage: string = i18n.global.t('errorParserUi.operationFailed')): string {
  defaultMessage = localizeLegacyFallback(defaultMessage)
  if (!err) return defaultMessage

  // 处理网络错误
  if (!isApiError(err) || !err.response) {
    if (err instanceof Error) {
      return normalizeKnownApiErrorMessage(err.message || defaultMessage)
    }
    return i18n.global.t('errorParserUi.network')
  }

  const data = err.response?.data

  // 1. 处理 {error: {type, message}} 格式（ProxyException 返回格式）
  if (data?.error?.message) {
    return normalizeKnownApiErrorMessage(data.error.message)
  }

  const detail = data?.detail

  // 如果没有 detail 字段
  if (!detail) {
    return normalizeKnownApiErrorMessage(data?.message || err.message || defaultMessage)
  }

  // 1. 处理 Pydantic 验证错误（数组格式）
  if (Array.isArray(detail)) {
    const errors = detail
      .map((error: ValidationError) => formatValidationError(error))
      .join('\n')
    return errors || defaultMessage
  }

  // 2. 处理字符串错误
  if (typeof detail === 'string') {
    return normalizeKnownApiErrorMessage(detail)
  }

  // 3. 处理对象错误
  if (typeof detail === 'object') {
    // 可能是自定义错误对象
    if ((detail as Record<string, unknown>).message) {
      return normalizeKnownApiErrorMessage(String((detail as Record<string, unknown>).message))
    }
    // 尝试 JSON 序列化
    try {
      return JSON.stringify(detail, null, 2)
    } catch {
      return defaultMessage
    }
  }

  return defaultMessage
}

/**
 * 解析并提取第一个错误信息（用于简短提示）
 */
export function parseApiErrorShort(err: unknown, defaultMessage: string = i18n.global.t('errorParserUi.operationFailed')): string {
  const fullError = parseApiError(err, defaultMessage)

  // 如果有多行错误，只取第一行
  const lines = fullError.split('\n')
  return lines[0] || defaultMessage
}

/**
 * 解析模型测试响应的错误信息
 * @param result 测试响应结果
 * @returns 格式化的错误信息
 */
export function parseTestModelError(result: {
  error?: string
  data?: {
    response?: {
      status_code?: number
      error?: string | { message?: string }
    }
  }
}): string {
  let errorMsg = result.error || i18n.global.t('errorParserUi.testFailed')

  // 检查HTTP状态码错误
  if (result.data?.response?.status_code) {
    const status = result.data.response.status_code
    if (status === 403) {
      errorMsg = i18n.global.t('errorParserUi.authInvalid')
    } else if (status === 401) {
      errorMsg = i18n.global.t('errorParserUi.authExpired')
    } else if (status === 404) {
      errorMsg = i18n.global.t('errorParserUi.modelMissing')
    } else if (status === 429) {
      errorMsg = i18n.global.t('errorParserUi.rateLimited')
    } else if (status >= 500) {
      errorMsg = i18n.global.t('errorParserUi.serverError', { status })
    } else {
      errorMsg = i18n.global.t('errorParserUi.requestFailed', { status })
    }
  }

  // 尝试从错误响应中提取更多信息
  if (result.data?.response?.error) {
    if (typeof result.data.response.error === 'string') {
      errorMsg = result.data.response.error
    } else if (result.data.response.error?.message) {
      errorMsg = result.data.response.error.message
    }
  }

  return errorMsg
}

/**
 * 解析上游模型查询错误信息
 * 将后端返回的原始错误信息（如 "HTTP 401: {json...}"）转换为友好的错误提示
 * @param error 错误字符串，格式可能是 "HTTP {status}: {json_body}" 或其他
 * @returns 友好的错误信息
 */
export function parseUpstreamModelError(error: string): string {
  if (!error) return i18n.global.t('errorParserUi.upstreamFetchFailed')

  // 匹配 "HTTP {status}: {body}" 格式
  const httpMatch = error.match(/^HTTP\s+(\d+):\s*(.*)$/s)
  if (httpMatch) {
    const status = parseInt(httpMatch[1], 10)
    const body = httpMatch[2]

    // 根据状态码生成友好消息
    let friendlyMsg = ''
    if (status === 401) {
      friendlyMsg = i18n.global.t('errorParserUi.keyInvalid')
    } else if (status === 403) {
      friendlyMsg = i18n.global.t('errorParserUi.keyForbidden')
    } else if (status === 404) {
      friendlyMsg = i18n.global.t('errorParserUi.modelListMissing')
    } else if (status === 429) {
      friendlyMsg = i18n.global.t('errorParserUi.rateLimited')
    } else if (status >= 500) {
      friendlyMsg = i18n.global.t('errorParserUi.upstreamUnavailable')
    }

    // 尝试从 JSON body 中提取更详细的错误信息
    if (body) {
      try {
        const parsed = JSON.parse(body)
        // 常见的错误格式: {error: {message: "..."}} 或 {error: "..."} 或 {message: "..."}
        let detailMsg = ''
        if (parsed.error?.message) {
          detailMsg = parsed.error.message
        } else if (typeof parsed.error === 'string') {
          detailMsg = parsed.error
        } else if (parsed.message) {
          detailMsg = parsed.message
        } else if (parsed.detail) {
          detailMsg = typeof parsed.detail === 'string' ? parsed.detail : JSON.stringify(parsed.detail)
        }

        // 如果提取到了详细消息，用它来丰富友好消息
        if (detailMsg) {
          // 检查是否是 token/认证相关的错误
          const lowerMsg = detailMsg.toLowerCase()
          if (lowerMsg.includes('invalid token') || lowerMsg.includes('invalid api key')) {
            return i18n.global.t('errorParserUi.keyInvalid')
          }
          if (lowerMsg.includes('expired')) {
            return i18n.global.t('errorParserUi.keyInvalid')
          }
          if (lowerMsg.includes('quota') || lowerMsg.includes('exceeded')) {
            return i18n.global.t('errorParserFeedback.quotaExceeded')
          }
          if (lowerMsg.includes('rate limit')) {
            return i18n.global.t('errorParserUi.rateLimited')
          }
          // 没有匹配特定关键词，但有详细信息，使用它作为补充
          if (friendlyMsg) {
            const truncated = detailMsg.length > 80 ? `${detailMsg.substring(0, 80)  }...` : detailMsg
            return `${friendlyMsg}: ${truncated}`
          }
          // 没有友好消息，直接使用详细信息
          const truncated = detailMsg.length > 100 ? `${detailMsg.substring(0, 100)  }...` : detailMsg
          return truncated
        }
      } catch {
        // JSON 解析失败，忽略
      }
    }

    // 返回友好消息，附加状态码
    if (friendlyMsg) {
      return friendlyMsg
    }
    return i18n.global.t('errorParserUi.requestFailed', { status })
  }

  // 检查是否是请求错误
  if (error.startsWith('Request error:')) {
    const detail = error.replace('Request error:', '').trim()
    if (detail.toLowerCase().includes('timeout')) {
      return i18n.global.t('errorParserUi.timeout')
    }
    if (detail.toLowerCase().includes('connection')) {
      return i18n.global.t('errorParserUi.connection')
    }
    return i18n.global.t('errorParserUi.networkRequest')
  }

  // 检查是否是未知 API 格式
  if (error.startsWith('Unknown API format:')) {
    return i18n.global.t('errorParserUi.unsupportedFormat')
  }

  // 如果包含分号，可能是多个错误合并的，取第一个
  if (error.includes('; ')) {
    const firstError = error.split('; ')[0]
    return parseUpstreamModelError(firstError)
  }

  // 默认返回原始错误（截断过长的部分）
  if (error.length > 100) {
    return `${error.substring(0, 100)  }...`
  }
  return error
}
