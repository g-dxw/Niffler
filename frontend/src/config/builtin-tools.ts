import { Mail, Shield, AlertTriangle } from 'lucide-vue-next'
import type { LucideIcon } from 'lucide-vue-next'
import { i18n } from '@/i18n'

export interface BuiltinTool {
  name: string
  description: string
  href: string
  icon: LucideIcon
}

export const BUILTIN_TOOLS: BuiltinTool[] = [
  {
    name: i18n.global.t('staticUi.emailConfig'),
    description: i18n.global.t('staticUi.emailConfigDesc'),
    href: '/admin/email',
    icon: Mail,
  },
  {
    name: i18n.global.t('staticUi.ipSecurity'),
    description: i18n.global.t('staticUi.ipSecurityDesc'),
    href: '/admin/ip-security',
    icon: Shield,
  },
  {
    name: i18n.global.t('staticUi.auditLogs'),
    description: i18n.global.t('staticUi.auditLogsDesc'),
    href: '/admin/audit-logs',
    icon: AlertTriangle,
  },
]

/** href → display name mapping for breadcrumbs */
export const BUILTIN_TOOL_BREADCRUMBS: Record<string, string> = Object.fromEntries(
  BUILTIN_TOOLS.map(t => [t.href, t.name])
)
