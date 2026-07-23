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

export const routingConditionFieldLabels: Record<string, string> = {
  model: i18n.global.t('routingUi.model'),
  api_format: i18n.global.t('routingUi.apiFormat'),
  user_id: i18n.global.t('routingUi.user'),
  api_key_id: 'API Key',
}

export const routingConditionOpLabels: Record<RoutingConditionOp, string> = {
  eq: i18n.global.t('routingUi.eq'),
  ne: i18n.global.t('routingUi.neq'),
  in: i18n.global.t('routingUi.in'),
  contains: i18n.global.t('routingUi.contains'),
  exists: i18n.global.t('routingUi.exists'),
  matches: i18n.global.t('routingUi.matches'),
}

export function isConditionLeaf(condition: RoutingCondition): condition is RoutingConditionLeaf {
  return typeof (condition as RoutingConditionLeaf).field === 'string'
}

export function summarizeRoutingCondition(condition: RoutingCondition): string {
  if (isConditionLeaf(condition)) {
    const field = routingConditionFieldLabels[condition.field] ?? condition.field
    const op = routingConditionOpLabels[condition.op] ?? condition.op
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
