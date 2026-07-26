import { afterEach, describe, expect, it } from 'vitest'

import { createAnalysisHoursOptions } from '@/composables/useTTLAnalysis'
import { createBuiltinTools } from '@/config/builtin-tools'
import { createDemoModeInfo } from '@/config/demo'
import { summarizeRoutingCondition } from '@/features/routing/utils/routingConditions'
import { i18n } from '@/i18n'
import { parseApiError } from '@/utils/errorParser'
import { getAnnouncementTypeLabel } from '@/utils/announcement'
import { createPasswordPolicyOptions } from '@/utils/passwordPolicy'
import { createNifflerServiceTemplates } from '@/views/admin/niffler-upstream-service-templates'

const t = i18n.global.t

function translatedSnapshot() {
  const validationError = {
    response: {
      data: {
        detail: [{ loc: ['body', 'api_key'], msg: 'required', type: 'missing' }],
      },
    },
  }

  return {
    builtinTool: createBuiltinTools(t)[0]?.name,
    ttlOption: createAnalysisHoursOptions(t)[0]?.label,
    routingCondition: summarizeRoutingCondition({ field: 'model', op: 'eq', value: 'gpt-5' }),
    fieldError: parseApiError(validationError),
    upstreamTemplate: createNifflerServiceTemplates(t)
      .find(template => template.key === 'openai_compatible')?.description,
    demoTitle: createDemoModeInfo(t).title,
    passwordPolicy: createPasswordPolicyOptions(t)[0]?.description,
    announcementType: getAnnouncementTypeLabel('important'),
  }
}

afterEach(() => {
  i18n.global.locale.value = 'zh-CN'
})

describe('dynamic module text', () => {
  it('rebuilds option, error, tool and template text after changing locale', () => {
    i18n.global.locale.value = 'zh-CN'
    const chinese = translatedSnapshot()

    i18n.global.locale.value = 'en-US'
    const english = translatedSnapshot()

    expect(chinese.builtinTool).not.toBe(english.builtinTool)
    expect(chinese.ttlOption).toBe('12 小时')
    expect(english.ttlOption).toBe('12 hours')
    expect(chinese.routingCondition).not.toBe(english.routingCondition)
    expect(chinese.fieldError).not.toBe(english.fieldError)
    expect(chinese.upstreamTemplate).not.toBe(english.upstreamTemplate)
    expect(chinese.demoTitle).not.toBe(english.demoTitle)
    expect(chinese.passwordPolicy).not.toBe(english.passwordPolicy)
    expect(chinese.announcementType).not.toBe(english.announcementType)
  })
})
