export type RoutingConditionOp = 'eq' | 'ne' | 'in' | 'contains' | 'exists' | 'matches'

export interface RoutingConditionLeaf {
  field: string
  op: RoutingConditionOp
  value?: unknown
}

export interface RoutingConditionGroup {
  all?: RoutingCondition[]
  any?: RoutingCondition[]
  not?: RoutingCondition
}

export type RoutingCondition = RoutingConditionLeaf | RoutingConditionGroup

export const routingConditionFieldLabelKeys: Record<string, string> = {
  model: 'routingUi.model',
  api_format: 'routingUi.apiFormat',
  user_id: 'routingUi.user',
}

export const routingConditionOpLabelKeys: Record<RoutingConditionOp, string> = {
  eq: 'routingUi.eq',
  ne: 'routingUi.neq',
  in: 'routingUi.in',
  contains: 'routingUi.contains',
  exists: 'routingUi.exists',
  matches: 'routingUi.matches',
}

export function isConditionLeaf(condition: RoutingCondition): condition is RoutingConditionLeaf {
  return typeof (condition as RoutingConditionLeaf).field === 'string'
}

export function summarizeRoutingCondition(condition: RoutingCondition): string {
  if (isConditionLeaf(condition)) {
    const fieldKey = routingConditionFieldLabelKeys[condition.field]
    const opKey = routingConditionOpLabelKeys[condition.op]
    const field = fieldKey ? i18n.global.t(fieldKey) : condition.field === 'api_key_id' ? 'API Key' : condition.field
    const op = opKey ? i18n.global.t(opKey) : condition.op
    return `${field} ${op} ${formatConditionValue(condition.value)}`
  }

  if (condition.all?.length) {
    return condition.all.map(summarizeRoutingCondition).join(` ${i18n.global.t('routingUi.and')} `)
  }

  if (condition.any?.length) {
    return condition.any.map(summarizeRoutingCondition).join(` ${i18n.global.t('routingUi.or')} `)
  }

  if (condition.not) {
    return `${i18n.global.t('routingUi.not')} ${summarizeRoutingCondition(condition.not)}`
  }

  return i18n.global.t('routingUi.noCondition')
}

function formatConditionValue(value: unknown): string {
  if (Array.isArray(value)) {
    return value.map(formatConditionValue).join(', ')
  }

  if (value === undefined || value === null) {
    return ''
  }

  if (typeof value === 'object') {
    return JSON.stringify(value)
  }

  return String(value)
}
import { i18n } from '@/i18n'
