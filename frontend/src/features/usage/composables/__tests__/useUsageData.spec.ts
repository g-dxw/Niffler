import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'

const {
  getAllUsageRecordsMock,
  getUsageStatsMock,
  getUsageByModelMock,
  getUsageByProviderMock,
  getUsageByApiFormatMock,
  meGetUsageMock,
} = vi.hoisted(() => ({
  getAllUsageRecordsMock: vi.fn(),
  getUsageStatsMock: vi.fn(),
  getUsageByModelMock: vi.fn(),
  getUsageByProviderMock: vi.fn(),
  getUsageByApiFormatMock: vi.fn(),
  meGetUsageMock: vi.fn(),
}))

vi.mock('@/api/usage', () => ({
  usageApi: {
    getAllUsageRecords: getAllUsageRecordsMock,
    getUsageStats: getUsageStatsMock,
    getUsageByModel: getUsageByModelMock,
    getUsageByProvider: getUsageByProviderMock,
    getUsageByApiFormat: getUsageByApiFormatMock,
  },
}))

vi.mock('@/api/me', () => ({
  meApi: {
    getUsage: meGetUsageMock,
  },
}))

vi.mock('@/utils/logger', () => ({
  log: {
    debug: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warn: vi.fn(),
    http: vi.fn(),
    performance: vi.fn(),
  },
}))

import { useUsageData } from '../useUsageData'
import type { UsageRecord } from '../../types'

function buildUsageRecord(overrides: Partial<UsageRecord> = {}): UsageRecord {
  return {
    id: 'usage-1',
    model: 'gpt-5',
    input_tokens: 10,
    output_tokens: 5,
    total_tokens: 15,
    cost: 0.01,
    is_stream: false,
    created_at: '2026-05-01T00:00:00Z',
    status: 'completed',
    ...overrides,
  }
}

describe('useUsageData', () => {
  beforeEach(() => {
    vi.clearAllMocks()

    getAllUsageRecordsMock.mockResolvedValue({
      records: [buildUsageRecord()],
      total: 1,
      limit: 20,
      offset: 0,
    })
    getUsageStatsMock.mockRejectedValue({
      response: { status: 500 },
      message: 'stats failed',
    })
    getUsageByModelMock.mockResolvedValue([])
    getUsageByProviderMock.mockResolvedValue([])
    getUsageByApiFormatMock.mockResolvedValue([])
    meGetUsageMock.mockResolvedValue({})
  })

  it('keeps admin records when stats refresh fails', async () => {
    const isAdminPage = ref(true)
    const { loadRecords, loadStats, currentRecords, totalRecords } = useUsageData({ isAdminPage })
    const dateRange = { preset: 'last7days', tz_offset_minutes: 0 }

    await loadRecords({ page: 1, pageSize: 20 }, undefined, dateRange)

    expect(currentRecords.value).toHaveLength(1)
    expect(totalRecords.value).toBe(1)

    await loadStats(dateRange)

    expect(currentRecords.value).toHaveLength(1)
    expect(currentRecords.value[0]).toMatchObject({
      id: 'usage-1',
      model: 'gpt-5',
    })
    expect(totalRecords.value).toBe(1)
  })

  it('passes the api key group filter when loading admin records', async () => {
    const isAdminPage = ref(true)
    const { loadRecords } = useUsageData({ isAdminPage })
    const dateRange = { preset: 'today', tz_offset_minutes: 480 }

    await loadRecords(
      { page: 2, pageSize: 50 },
      {
        user_id: 'user-1',
        api_key_group_id: 'group-claude',
        model: 'claude-sonnet-4-6',
      },
      dateRange
    )

    expect(getAllUsageRecordsMock).toHaveBeenCalledWith({
      preset: 'today',
      tz_offset_minutes: 480,
      limit: 50,
      offset: 50,
      user_id: 'user-1',
      api_key_group_id: 'group-claude',
      model: 'claude-sonnet-4-6',
    })
  })

  it('continues loading admin breakdowns when the summary request fails', async () => {
    const isAdminPage = ref(true)
    const {
      loadStats,
      stats,
      modelStats,
      providerStats,
      apiFormatStats,
      availableModels,
      availableProviders,
    } = useUsageData({ isAdminPage })
    const dateRange = { preset: 'last7days', tz_offset_minutes: 0 }

    getUsageStatsMock.mockRejectedValueOnce({
      response: { status: 500 },
      message: 'summary failed',
    })
    getUsageByModelMock.mockResolvedValueOnce([
      {
        model: 'gpt-5',
        request_count: 3,
        total_tokens: 300,
        official_cost: 2.34,
        total_cost: 1.23,
      },
    ])
    getUsageByProviderMock.mockResolvedValueOnce([
      {
        provider: 'OpenAI',
        request_count: 3,
        total_tokens: 300,
        total_cost: 1.23,
        actual_cost: 1.5,
        avg_response_time_ms: 1250,
        success_rate: 1,
      },
    ])
    getUsageByApiFormatMock.mockResolvedValueOnce([
      {
        api_format: 'openai:chat',
        request_count: 3,
        total_tokens: 300,
        total_cost: 1.23,
        actual_cost: 1.5,
        avg_response_time_ms: 1250,
      },
    ])

    const hadFailure = await loadStats(dateRange)

    expect(hadFailure).toBe(true)
    expect(stats.value).toMatchObject({
      total_requests: 0,
      total_tokens: 0,
      total_cost: 0,
    })
    expect(modelStats.value).toHaveLength(1)
    expect(modelStats.value[0].official_cost).toBe(2.34)
    expect(providerStats.value).toHaveLength(1)
    expect(apiFormatStats.value).toHaveLength(1)
    expect(availableModels.value).toEqual(['gpt-5'])
    expect(availableProviders.value).toEqual(['OpenAI'])
  })

  it('filters placeholder providers from admin provider stats', async () => {
    const isAdminPage = ref(true)
    const { loadStats, providerStats, availableProviders } = useUsageData({ isAdminPage })
    const dateRange = { preset: 'last7days', tz_offset_minutes: 0 }

    getUsageStatsMock.mockResolvedValueOnce({
      total_requests: 4,
      total_tokens: 400,
      total_cost: 1,
      avg_response_time: 0,
    })
    getUsageByProviderMock.mockResolvedValueOnce([
      {
        provider: 'OpenAI',
        request_count: 3,
        total_tokens: 300,
        total_cost: 1.23,
        actual_cost: 1.5,
        avg_response_time_ms: 1250,
        success_rate: 100,
      },
      {
        provider: 'Unknown',
        request_count: 1,
        total_tokens: 100,
        total_cost: 0,
        actual_cost: 0,
        avg_response_time_ms: 0,
        success_rate: 100,
      },
      {
        provider: 'unknow',
        request_count: 1,
        total_tokens: 100,
        total_cost: 0,
        actual_cost: 0,
        avg_response_time_ms: 0,
        success_rate: 100,
      },
      {
        provider: 'pending',
        request_count: 1,
        total_tokens: 100,
        total_cost: 0,
        actual_cost: 0,
        avg_response_time_ms: 0,
        success_rate: 100,
      },
    ])

    await loadStats(dateRange)

    expect(providerStats.value.map(item => item.provider)).toEqual(['OpenAI'])
    expect(availableProviders.value).toEqual(['OpenAI'])
  })

  it('uses refreshed fallback flags even when preserving a newer local status', async () => {
    const isAdminPage = ref(true)
    const { loadRecords, currentRecords } = useUsageData({ isAdminPage })
    const dateRange = { preset: 'last7days', tz_offset_minutes: 0 }

    getAllUsageRecordsMock
      .mockResolvedValueOnce({
        records: [
          buildUsageRecord({
            status: 'completed',
            has_fallback: true,
            provider: 'https://c-api.cc/',
            cost: 0,
          }),
        ],
        total: 1,
        limit: 20,
        offset: 0,
      })
      .mockResolvedValueOnce({
        records: [
          buildUsageRecord({
            status: 'streaming',
            has_fallback: false,
            provider: 'unknown',
            cost: 0.00326455,
            charge_breakdown: {
              official_cost: 0.065291,
              package_debit: 0,
              wallet_debit: 0.00326455,
              wallet_multiplier: 0.05,
              user_debit: 0.00326455,
            },
          }),
        ],
        total: 1,
        limit: 20,
        offset: 0,
      })

    await loadRecords({ page: 1, pageSize: 20 }, undefined, dateRange)
    await loadRecords({ page: 1, pageSize: 20 }, undefined, dateRange)

    expect(currentRecords.value[0].status).toBe('completed')
    expect(currentRecords.value[0].provider).toBe('https://c-api.cc/')
    expect(currentRecords.value[0].has_fallback).toBe(false)
    expect(currentRecords.value[0].charge_breakdown?.wallet_debit).toBe(0.00326455)
  })

  it('loads current-user records with backend pagination and filters', async () => {
    const isAdminPage = ref(false)
    const { loadRecords, currentRecords, totalRecords } = useUsageData({ isAdminPage })
    const dateRange = { preset: 'today', tz_offset_minutes: 480 }

    meGetUsageMock.mockResolvedValueOnce({
      records: [buildUsageRecord({ id: 'usage-remember-1', model: 'gpt-5.5' })],
      pagination: {
        total: 11214,
        limit: 50,
        offset: 100,
        has_more: true,
      },
    })

    await loadRecords(
      { page: 3, pageSize: 50 },
      {
        search: ' cc专用 ',
        model: 'gpt-5.5',
        api_format: 'openai:responses',
        status: 'failed',
        client_family: 'codex',
      },
      dateRange
    )

    expect(meGetUsageMock).toHaveBeenCalledWith({
      preset: 'today',
      tz_offset_minutes: 480,
      limit: 50,
      offset: 100,
      search: 'cc专用',
      model: 'gpt-5.5',
      api_format: 'openai:responses',
      status: 'failed',
      client_family: 'codex',
    })
    expect(currentRecords.value).toHaveLength(1)
    expect(currentRecords.value[0].id).toBe('usage-remember-1')
    expect(totalRecords.value).toBe(11214)
  })

  it('loads current-user stats without requesting records', async () => {
    const isAdminPage = ref(false)
    const { loadStats, stats, currentRecords, availableModels, modelStats } = useUsageData({ isAdminPage })
    const dateRange = { preset: 'today', tz_offset_minutes: 480 }

    meGetUsageMock.mockResolvedValueOnce({
      total_requests: 3,
      total_tokens: 300,
      total_cost: 0.12,
      avg_response_time: 1.2,
      summary_by_model: [
        {
          model: 'gpt-5.5',
          requests: 3,
          total_tokens: 300,
          official_cost: 0.09,
          total_cost_usd: 0.12,
        },
      ],
      records: [buildUsageRecord({ id: 'should-not-be-used' })],
    })

    await loadStats(dateRange)

    expect(meGetUsageMock).toHaveBeenCalledWith({
      preset: 'today',
      tz_offset_minutes: 480,
      include_records: false,
    })
    expect(stats.value.total_requests).toBe(3)
    expect(availableModels.value).toEqual(['gpt-5.5'])
    expect(modelStats.value[0].official_cost).toBe(0.09)
    expect(currentRecords.value).toEqual([])
  })
})
