import { describe, expect, it } from 'vitest'

import type { ProviderWithEndpointsSummary } from '@/api/endpoints/types'
import { useUserAccessControlOptions } from '@/features/users/composables/useUserAccessControlOptions'

function providerOptionProvider(
  id: string,
  name: string,
  is_active: boolean,
): ProviderWithEndpointsSummary {
  return {
    id,
    name,
    provider_priority: 0,
    keep_priority_on_conversion: false,
    enable_format_conversion: false,
    is_active,
    total_endpoints: 0,
    active_endpoints: 0,
    total_keys: 0,
    active_keys: 0,
    total_models: 0,
    active_models: 0,
    global_model_ids: [],
    avg_health_score: 0,
  } as ProviderWithEndpointsSummary
}

describe('useUserAccessControlOptions', () => {
  it('only exposes active providers for user group access control', () => {
    const { providers, providerOptions } = useUserAccessControlOptions()

    providers.value = [
      providerOptionProvider('active-provider', '启用服务', true),
      providerOptionProvider('inactive-provider', '停用服务', false),
    ]

    expect(providerOptions.value).toEqual([
      {
        value: 'active-provider',
        label: '启用服务',
      },
    ])
  })
})
