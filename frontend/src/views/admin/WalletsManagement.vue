<template>
  <div class="space-y-6 pb-8">
    <Card class="overflow-hidden">
      <div class="px-5 py-4 border-b border-border/60 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h3 class="text-base font-semibold">
            钱包管理
          </h3>
          <p class="text-xs text-muted-foreground mt-1">
            统一管理资金流水、退款审批、支付订单与支付回调
          </p>
        </div>
      </div>

      <div class="px-5 py-5">
        <Tabs v-model="activeTab">
          <TabsList class="tabs-button-list grid w-full max-w-[960px] grid-cols-2 sm:grid-cols-5">
            <TabsTrigger value="ledger">
              资金流水
            </TabsTrigger>
            <TabsTrigger value="orders">
              支付订单
            </TabsTrigger>
            <TabsTrigger value="refunds">
              退款审批
            </TabsTrigger>
            <TabsTrigger value="callbacks">
              回调日志
            </TabsTrigger>
            <TabsTrigger value="redeem_codes">
              兑换码
            </TabsTrigger>
          </TabsList>

          <TabsContent
            value="ledger"
            class="mt-5 space-y-4"
          >
            <div class="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
              <div class="flex flex-wrap items-center gap-2">
                <Select v-model="ledgerCategoryFilter">
                  <SelectTrigger class="w-[170px]">
                    <SelectValue placeholder="一级分类" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">
                      全部分类
                    </SelectItem>
                    <SelectItem value="recharge">
                      充值
                    </SelectItem>
                    <SelectItem value="gift">
                      赠款
                    </SelectItem>
                    <SelectItem value="adjust">
                      调账
                    </SelectItem>
                    <SelectItem value="refund">
                      退款
                    </SelectItem>
                  </SelectContent>
                </Select>

                <Select v-model="ledgerReasonFilter">
                  <SelectTrigger class="w-[180px]">
                    <SelectValue placeholder="二级分类" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">
                      全部二级
                    </SelectItem>
                    <SelectItem
                      v-for="option in ledgerReasonOptions"
                      :key="option.value"
                      :value="option.value"
                    >
                      {{ option.label }}
                    </SelectItem>
                  </SelectContent>
                </Select>

                <Select v-model="ledgerOwnerFilter">
                  <SelectTrigger class="w-[170px]">
                    <SelectValue placeholder="归属类型" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">
                      全部归属
                    </SelectItem>
                    <SelectItem value="user">
                      用户钱包
                    </SelectItem>
                    <SelectItem value="api_key">
                      独立密钥钱包
                    </SelectItem>
                  </SelectContent>
                </Select>

                <Input
                  v-model="ledgerUserSearch"
                  type="search"
                  class="w-[260px]"
                  placeholder="搜索用户名 / 邮箱 / 用户ID"
                />
              </div>

              <div class="flex items-center justify-between gap-3">
                <div class="text-sm text-muted-foreground">
                  共 {{ ledgerTotal }} 条
                </div>
                <RefreshButton
                  :loading="loadingLedger"
                  @click="loadLedger"
                />
              </div>
            </div>

            <div class="rounded-2xl border border-border/60 overflow-hidden bg-background">
              <div class="overflow-x-auto">
                <Table class="w-full min-w-[1200px] table-fixed">
                  <colgroup>
                    <col :style="{ width: ledgerTableColumnWidths.time }">
                    <col :style="{ width: ledgerTableColumnWidths.owner }">
                    <col :style="{ width: ledgerTableColumnWidths.type }">
                    <col :style="{ width: ledgerTableColumnWidths.amount }">
                    <col :style="{ width: ledgerTableColumnWidths.balance }">
                    <col :style="{ width: ledgerTableColumnWidths.description }">
                    <col :style="{ width: ledgerTableColumnWidths.actions }">
                  </colgroup>
                  <TableHeader>
                    <TableRow>
                      <SortableTableHead
                        :sortable="false"
                        resize-column-key="time"
                        :resizable="true"
                        @resize-start="handleLedgerTableColumnResizeStart"
                      >
                        时间
                      </SortableTableHead>
                      <SortableTableHead
                        :sortable="false"
                        resize-column-key="owner"
                        :resizable="true"
                        @resize-start="handleLedgerTableColumnResizeStart"
                      >
                        归属
                      </SortableTableHead>
                      <SortableTableHead
                        :sortable="false"
                        resize-column-key="type"
                        :resizable="true"
                        @resize-start="handleLedgerTableColumnResizeStart"
                      >
                        类型
                      </SortableTableHead>
                      <SortableTableHead
                        :sortable="false"
                        resize-column-key="amount"
                        :resizable="true"
                        @resize-start="handleLedgerTableColumnResizeStart"
                      >
                        金额
                      </SortableTableHead>
                      <SortableTableHead
                        :sortable="false"
                        resize-column-key="balance"
                        :resizable="true"
                        @resize-start="handleLedgerTableColumnResizeStart"
                      >
                        余额变化
                      </SortableTableHead>
                      <SortableTableHead
                        :sortable="false"
                        resize-column-key="description"
                        :resizable="true"
                        @resize-start="handleLedgerTableColumnResizeStart"
                      >
                        说明
                      </SortableTableHead>
                      <SortableTableHead
                        class="text-right"
                        :sortable="false"
                        align="right"
                        resize-column-key="actions"
                        :resizable="true"
                        @resize-start="handleLedgerTableColumnResizeStart"
                      >
                        操作
                      </SortableTableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow
                      v-for="tx in ledgerItems"
                      :key="tx.id"
                      class="hover:bg-muted/20"
                    >
                      <TableCell class="text-xs text-muted-foreground whitespace-nowrap">
                        {{ formatDateTime(tx.created_at) }}
                      </TableCell>
                      <TableCell class="min-w-[180px]">
                        <div class="font-medium text-sm">
                          {{ ownerDisplayName(tx.owner_name, tx.owner_type) }}
                        </div>
                        <div class="text-xs text-muted-foreground mt-1 flex items-center gap-2">
                          <span>{{ ownerTypeLabel(tx.owner_type) }}</span>
                          <Badge
                            v-if="tx.wallet_status"
                            variant="outline"
                            class="text-[10px]"
                          >
                            {{ walletStatusLabel(tx.wallet_status) }}
                          </Badge>
                        </div>
                      </TableCell>
                      <TableCell>
                        <div class="space-y-1">
                          <Badge
                            variant="outline"
                            class="font-mono"
                          >
                            {{ walletTransactionCategoryLabel(tx.category) }}
                          </Badge>
                          <div class="text-[11px] text-muted-foreground">
                            {{ walletTransactionReasonLabel(tx.reason_code) }}
                          </div>
                        </div>
                      </TableCell>
                      <TableCell :class="tx.amount >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'">
                        {{ tx.amount >= 0 ? '+' : '' }}{{ tx.amount.toFixed(4) }}
                      </TableCell>
                      <TableCell class="text-xs tabular-nums whitespace-nowrap">
                        <div>{{ tx.balance_before.toFixed(4) }} → {{ tx.balance_after.toFixed(4) }}</div>
                        <div
                          v-if="tx.recharge_balance_before !== null && tx.recharge_balance_before !== undefined && tx.gift_balance_before !== null && tx.gift_balance_before !== undefined"
                          class="text-[11px] text-muted-foreground mt-0.5"
                        >
                          充 {{ Number(tx.recharge_balance_before).toFixed(4) }}→{{ Number(tx.recharge_balance_after ?? 0).toFixed(4) }}
                          · 赠 {{ Number(tx.gift_balance_before).toFixed(4) }}→{{ Number(tx.gift_balance_after ?? 0).toFixed(4) }}
                        </div>
                      </TableCell>
                      <TableCell
                        class="text-xs text-muted-foreground whitespace-pre-wrap break-words"
                        :title="tx.description || '-'"
                      >
                        {{ tx.description || '-' }}
                      </TableCell>
                      <TableCell class="text-right">
                        <Button
                          size="sm"
                          variant="outline"
                          @click="openLedgerDrawer(tx)"
                        >
                          详情
                        </Button>
                      </TableCell>
                    </TableRow>
                    <TableRow v-if="!loadingLedger && ledgerItems.length === 0">
                      <TableCell
                        colspan="7"
                        class="py-12"
                      >
                        <EmptyState
                          title="暂无资金流水"
                          description="当前筛选条件下没有资金动作记录"
                        />
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </div>
            </div>

            <Pagination
              :current="ledgerPage"
              :total="ledgerTotal"
              :page-size="ledgerPageSize"
              @update:current="handleLedgerPageChange"
              @update:page-size="handleLedgerPageSizeChange"
            />
          </TabsContent>

          <TabsContent
            value="refunds"
            class="mt-5 space-y-4"
          >
            <div class="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
              <div class="flex flex-wrap items-center gap-2">
                <Select v-model="refundStatusFilter">
                  <SelectTrigger class="w-[170px]">
                    <SelectValue placeholder="退款状态" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">
                      全部状态
                    </SelectItem>
                    <SelectItem value="pending_approval">
                      待审批
                    </SelectItem>
                    <SelectItem value="approved">
                      已审批
                    </SelectItem>
                    <SelectItem value="processing">
                      处理中
                    </SelectItem>
                    <SelectItem value="succeeded">
                      已完成
                    </SelectItem>
                    <SelectItem value="failed">
                      已失败
                    </SelectItem>
                  </SelectContent>
                </Select>

                <Select v-model="refundOwnerFilter">
                  <SelectTrigger class="w-[170px]">
                    <SelectValue placeholder="归属类型" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">
                      全部归属
                    </SelectItem>
                    <SelectItem value="user">
                      用户钱包
                    </SelectItem>
                  </SelectContent>
                </Select>

                <Input
                  v-model="refundUserSearch"
                  type="search"
                  class="w-[260px]"
                  placeholder="搜索用户名 / 邮箱 / 用户ID"
                />
              </div>

              <div class="flex items-center justify-between gap-3">
                <div class="text-sm text-muted-foreground">
                  共 {{ refundTotal }} 条
                </div>
                <RefreshButton
                  :loading="loadingRefunds"
                  @click="loadRefunds"
                />
              </div>
            </div>

            <div class="rounded-2xl border border-border/60 overflow-hidden bg-background">
              <div class="overflow-x-auto">
                <Table class="w-full min-w-[1260px] table-fixed">
                  <colgroup>
                    <col :style="{ width: refundTableColumnWidths.owner }">
                    <col :style="{ width: refundTableColumnWidths.refundNo }">
                    <col :style="{ width: refundTableColumnWidths.amount }">
                    <col :style="{ width: refundTableColumnWidths.mode }">
                    <col :style="{ width: refundTableColumnWidths.status }">
                    <col :style="{ width: refundTableColumnWidths.reason }">
                    <col :style="{ width: refundTableColumnWidths.created }">
                    <col :style="{ width: refundTableColumnWidths.actions }">
                  </colgroup>
                  <TableHeader>
                    <TableRow>
                      <SortableTableHead :sortable="false" resize-column-key="owner" :resizable="true" @resize-start="handleRefundTableColumnResizeStart">
                        归属
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="refundNo" :resizable="true" @resize-start="handleRefundTableColumnResizeStart">
                        退款单号
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="amount" :resizable="true" @resize-start="handleRefundTableColumnResizeStart">
                        金额
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="mode" :resizable="true" @resize-start="handleRefundTableColumnResizeStart">
                        模式
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="status" :resizable="true" @resize-start="handleRefundTableColumnResizeStart">
                        状态
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="reason" :resizable="true" @resize-start="handleRefundTableColumnResizeStart">
                        原因
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="created" :resizable="true" @resize-start="handleRefundTableColumnResizeStart">
                        申请时间
                      </SortableTableHead>
                      <SortableTableHead class="text-right" :sortable="false" align="right" resize-column-key="actions" :resizable="true" @resize-start="handleRefundTableColumnResizeStart">
                        操作
                      </SortableTableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow
                      v-for="refund in refundItems"
                      :key="refund.id"
                      class="hover:bg-muted/20"
                    >
                      <TableCell class="min-w-[180px]">
                        <div class="font-medium text-sm">
                          {{ ownerDisplayName(refund.owner_name, refund.owner_type) }}
                        </div>
                        <div class="text-xs text-muted-foreground mt-1 flex items-center gap-2">
                          <span>{{ ownerTypeLabel(refund.owner_type) }}</span>
                          <Badge
                            v-if="refund.wallet_status"
                            variant="outline"
                            class="text-[10px]"
                          >
                            {{ walletStatusLabel(refund.wallet_status) }}
                          </Badge>
                        </div>
                      </TableCell>
                      <TableCell class="font-mono text-xs whitespace-nowrap">
                        {{ refund.refund_no }}
                      </TableCell>
                      <TableCell class="tabular-nums whitespace-nowrap">
                        {{ formatCurrency(refund.amount_usd) }}
                      </TableCell>
                      <TableCell>
                        {{ refundModeLabel(refund.refund_mode) }}
                      </TableCell>
                      <TableCell>
                        <Badge :variant="refundStatusBadge(refund.status)">
                          {{ refundStatusLabel(refund.status) }}
                        </Badge>
                      </TableCell>
                      <TableCell
                        class="text-xs text-muted-foreground whitespace-pre-wrap break-words"
                        :title="refund.reason || refund.failure_reason || '-'"
                      >
                        {{ refund.reason || refund.failure_reason || '-' }}
                      </TableCell>
                      <TableCell class="text-xs text-muted-foreground whitespace-nowrap">
                        {{ formatDateTime(refund.created_at) }}
                      </TableCell>
                      <TableCell class="text-right">
                        <div class="flex justify-end gap-2">
                          <Button
                            size="sm"
                            variant="outline"
                            @click="openRefundDrawer(refund)"
                          >
                            审批
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                    <TableRow v-if="!loadingRefunds && refundItems.length === 0">
                      <TableCell
                        colspan="8"
                        class="py-12"
                      >
                        <EmptyState
                          title="暂无退款申请"
                          description="当前筛选条件下没有退款单"
                        />
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </div>
            </div>

            <Pagination
              :current="refundPage"
              :total="refundTotal"
              :page-size="refundPageSize"
              @update:current="handleRefundPageChange"
              @update:page-size="handleRefundPageSizeChange"
            />
          </TabsContent>

          <TabsContent
            value="orders"
            class="mt-5 space-y-4"
          >
            <div class="flex flex-wrap items-center gap-2">
              <Select v-model="orderStatusFilter">
                <SelectTrigger class="w-[180px]">
                  <SelectValue placeholder="订单状态" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">
                    全部状态
                  </SelectItem>
                  <SelectItem value="pending">
                    待支付
                  </SelectItem>
                  <SelectItem value="paid">
                    已支付
                  </SelectItem>
                  <SelectItem value="credited">
                    已到账
                  </SelectItem>
                  <SelectItem value="failed">
                    支付失败
                  </SelectItem>
                  <SelectItem value="expired">
                    已过期
                  </SelectItem>
                </SelectContent>
              </Select>

              <Select v-model="orderKindFilter">
                <SelectTrigger class="w-[180px]">
                  <SelectValue placeholder="订单内容" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">
                    全部订单
                  </SelectItem>
                  <SelectItem value="wallet_recharge">
                    钱包充值
                  </SelectItem>
                  <SelectItem value="plan_purchase">
                    套餐购买
                  </SelectItem>
                </SelectContent>
              </Select>

              <Select v-model="orderMethodFilter">
                <SelectTrigger class="w-[180px]">
                  <SelectValue placeholder="支付方式" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">
                    全部方式
                  </SelectItem>
                  <SelectItem value="alipay">
                    支付宝
                  </SelectItem>
                  <SelectItem value="wechat">
                    微信支付
                  </SelectItem>
                  <SelectItem value="admin_manual">
                    人工充值
                  </SelectItem>
                  <SelectItem value="card_code">
                    充值卡
                  </SelectItem>
                  <SelectItem value="gift_code">
                    礼品卡
                  </SelectItem>
                  <SelectItem value="card_recharge">
                    卡密充值
                  </SelectItem>
                </SelectContent>
              </Select>

              <Input
                v-model="orderUserSearch"
                type="search"
                class="w-[260px]"
                placeholder="搜索用户名 / 邮箱 / 用户ID"
              />

              <RefreshButton
                :loading="loadingOrders"
                @click="loadOrders"
              />
            </div>

            <div class="rounded-2xl border border-border/60 overflow-hidden bg-background">
              <div class="overflow-x-auto">
                <Table class="w-full min-w-[1220px] table-fixed">
                  <colgroup>
                    <col :style="{ width: paymentOrderTableColumnWidths.orderNo }">
                    <col :style="{ width: paymentOrderTableColumnWidths.wallet }">
                    <col :style="{ width: paymentOrderTableColumnWidths.content }">
                    <col :style="{ width: paymentOrderTableColumnWidths.amount }">
                    <col :style="{ width: paymentOrderTableColumnWidths.method }">
                    <col :style="{ width: paymentOrderTableColumnWidths.status }">
                    <col :style="{ width: paymentOrderTableColumnWidths.created }">
                    <col :style="{ width: paymentOrderTableColumnWidths.actions }">
                  </colgroup>
                  <TableHeader>
                    <TableRow>
                      <SortableTableHead :sortable="false" resize-column-key="orderNo" :resizable="true" @resize-start="handlePaymentOrderTableColumnResizeStart">
                        订单号
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="wallet" :resizable="true" @resize-start="handlePaymentOrderTableColumnResizeStart">
                        钱包名称
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="content" :resizable="true" @resize-start="handlePaymentOrderTableColumnResizeStart">
                        订单内容
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="amount" :resizable="true" @resize-start="handlePaymentOrderTableColumnResizeStart">
                        金额
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="method" :resizable="true" @resize-start="handlePaymentOrderTableColumnResizeStart">
                        支付方式
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="status" :resizable="true" @resize-start="handlePaymentOrderTableColumnResizeStart">
                        状态
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="created" :resizable="true" @resize-start="handlePaymentOrderTableColumnResizeStart">
                        创建时间
                      </SortableTableHead>
                      <SortableTableHead class="text-right" :sortable="false" align="right" resize-column-key="actions" :resizable="true" @resize-start="handlePaymentOrderTableColumnResizeStart">
                        操作
                      </SortableTableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow
                      v-for="order in orders"
                      :key="order.id"
                    >
                      <TableCell
                        class="font-mono text-xs break-all"
                        :title="order.order_no"
                      >
                        {{ order.order_no }}
                      </TableCell>
                      <TableCell>
                        <div class="text-sm font-medium">
                          {{ orderWalletName(order.wallet_id) }}
                        </div>
                        <div class="text-xs text-muted-foreground mt-1">
                          {{ orderWalletTypeLabel(order.wallet_id) }}
                        </div>
                      </TableCell>
                      <TableCell>
                        <div class="text-sm font-medium truncate" :title="paymentOrderContentLabel(order)">
                          {{ paymentOrderContentLabel(order) }}
                        </div>
                        <div v-if="order.order_kind === 'plan_purchase' && order.fulfillment_status" class="text-xs text-muted-foreground mt-1">
                          {{ paymentOrderFulfillmentLabel(order.fulfillment_status) }}
                        </div>
                      </TableCell>
                      <TableCell class="tabular-nums">
                        {{ formatCurrency(order.amount_usd) }}
                      </TableCell>
                      <TableCell>{{ paymentOrderMethodLabel(order) }}</TableCell>
                      <TableCell>
                        <Badge :variant="paymentStatusBadge(order.status)">
                          {{ paymentStatusLabel(order.status) }}
                        </Badge>
                      </TableCell>
                      <TableCell class="text-xs text-muted-foreground whitespace-nowrap">
                        {{ formatDateTime(order.created_at) }}
                      </TableCell>
                      <TableCell class="text-right">
                        <div class="flex justify-end gap-2">
                          <Button
                            v-if="canCreditOrder(order.status)"
                            size="sm"
                            @click="openCreditDialog(order)"
                          >
                            {{ paymentOrderCreditActionLabel(order) }}
                          </Button>
                          <Button
                            v-if="canExpireOrder(order.status)"
                            size="sm"
                            variant="outline"
                            :disabled="submittingOrderAction"
                            @click="expireOrder(order.id)"
                          >
                            过期
                          </Button>
                          <Button
                            v-if="canFailOrder(order.status)"
                            size="sm"
                            variant="destructive"
                            :disabled="submittingOrderAction"
                            @click="failOrder(order.id)"
                          >
                            失败
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                    <TableRow v-if="!loadingOrders && orders.length === 0">
                      <TableCell
                        colspan="8"
                        class="py-10"
                      >
                        <EmptyState
                          title="暂无支付订单"
                          description="当前筛选条件下没有数据"
                        />
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </div>
            </div>

            <Pagination
              :current="orderPage"
              :total="orderTotal"
              :page-size="orderPageSize"
              @update:current="handleOrderPageChange"
              @update:page-size="handleOrderPageSizeChange"
            />
          </TabsContent>

          <TabsContent
            value="callbacks"
            class="mt-5 space-y-4"
          >
            <div class="flex items-center gap-2">
              <Select v-model="callbackMethodFilter">
                <SelectTrigger class="w-[180px]">
                  <SelectValue placeholder="支付方式" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">
                    全部方式
                  </SelectItem>
                  <SelectItem value="alipay">
                    支付宝
                  </SelectItem>
                  <SelectItem value="wechat">
                    微信支付
                  </SelectItem>
                </SelectContent>
              </Select>
              <RefreshButton
                :loading="loadingCallbacks"
                @click="loadCallbacks"
              />
            </div>

            <div class="rounded-2xl border border-border/60 overflow-hidden bg-background">
              <div class="overflow-x-auto">
                <Table class="w-full min-w-[920px] table-fixed">
                  <colgroup>
                    <col :style="{ width: callbackTableColumnWidths.callbackKey }">
                    <col :style="{ width: callbackTableColumnWidths.orderNo }">
                    <col :style="{ width: callbackTableColumnWidths.method }">
                    <col :style="{ width: callbackTableColumnWidths.signature }">
                    <col :style="{ width: callbackTableColumnWidths.status }">
                    <col :style="{ width: callbackTableColumnWidths.time }">
                  </colgroup>
                  <TableHeader>
                    <TableRow>
                      <SortableTableHead :sortable="false" resize-column-key="callbackKey" :resizable="true" @resize-start="handleCallbackTableColumnResizeStart">
                        回调键
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="orderNo" :resizable="true" @resize-start="handleCallbackTableColumnResizeStart">
                        订单号
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="method" :resizable="true" @resize-start="handleCallbackTableColumnResizeStart">
                        方式
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="signature" :resizable="true" @resize-start="handleCallbackTableColumnResizeStart">
                        验签
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="status" :resizable="true" @resize-start="handleCallbackTableColumnResizeStart">
                        状态
                      </SortableTableHead>
                      <SortableTableHead :sortable="false" resize-column-key="time" :resizable="true" @resize-start="handleCallbackTableColumnResizeStart">
                        时间
                      </SortableTableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow
                      v-for="callback in callbacks"
                      :key="callback.id"
                    >
                      <TableCell
                        class="font-mono text-xs break-all"
                        :title="callback.callback_key"
                      >
                        {{ callback.callback_key }}
                      </TableCell>
                      <TableCell
                        class="font-mono text-xs break-all"
                        :title="callback.order_no || '-'"
                      >
                        {{ callback.order_no || '-' }}
                      </TableCell>
                      <TableCell>{{ paymentMethodLabel(callback.payment_method) }}</TableCell>
                      <TableCell>
                        <Badge :variant="callback.signature_valid ? 'success' : 'destructive'">
                          {{ callback.signature_valid ? '通过' : '失败' }}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <Badge :variant="callbackStatusBadge(callback.status)">
                          {{ callbackStatusLabel(callback.status) }}
                        </Badge>
                      </TableCell>
                      <TableCell class="text-xs text-muted-foreground whitespace-nowrap">
                        {{ formatDateTime(callback.created_at) }}
                      </TableCell>
                    </TableRow>
                    <TableRow v-if="!loadingCallbacks && callbacks.length === 0">
                      <TableCell
                        colspan="6"
                        class="py-10"
                      >
                        <EmptyState
                          title="暂无回调日志"
                          description="当前筛选条件下没有数据"
                        />
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </div>
            </div>

            <Pagination
              :current="callbackPage"
              :total="callbackTotal"
              :page-size="callbackPageSize"
              @update:current="handleCallbackPageChange"
              @update:page-size="handleCallbackPageSizeChange"
            />
          </TabsContent>

          <TabsContent
            value="redeem_codes"
            class="mt-5 space-y-5"
          >
            <div class="rounded-2xl border border-border/60 bg-background p-4 space-y-4">
              <div class="flex items-center justify-between gap-3">
                <div>
                  <h4 class="text-sm font-semibold">
                    批量生成兑换码
                  </h4>
                  <p class="text-xs text-muted-foreground mt-1">
                    生成后本会话可切换显示明文；页面刷新后仅保留脱敏码。
                  </p>
                </div>
                <RefreshButton
                  :loading="loadingRedeemBatches || loadingRedeemCodes"
                  @click="loadRedeemCodeBatches"
                />
              </div>

              <div class="grid gap-3 lg:grid-cols-4">
                <div class="space-y-1.5">
                  <Label>批次名称</Label>
                  <Input v-model="redeemBatchForm.name" />
                </div>
                <div class="space-y-1.5">
                  <Label>面额 (USD)</Label>
                  <Input
                    v-model.number="redeemBatchForm.amount_usd"
                    type="number"
                    min="0.01"
                    step="0.01"
                  />
                </div>
                <div class="space-y-1.5">
                  <Label>生成数量</Label>
                  <Input
                    v-model.number="redeemBatchForm.total_count"
                    type="number"
                    min="1"
                    step="1"
                  />
                </div>
                <div class="space-y-1.5">
                  <Label>过期时间（可选）</Label>
                  <Input
                    v-model="redeemBatchForm.expires_at"
                    type="datetime-local"
                  />
                </div>
              </div>

              <div class="space-y-1.5">
                <Label>备注（可选）</Label>
                <Textarea
                  v-model="redeemBatchForm.description"
                  rows="3"
                  placeholder="例如：五一活动 / 线下渠道 / KOC 发放"
                />
              </div>

              <div class="flex flex-wrap justify-end gap-2">
                <Button
                  variant="outline"
                  :disabled="!canExportLatestGeneratedRedeemCodes"
                  @click="exportLatestGeneratedRedeemCodes"
                >
                  导出最近生成
                </Button>
                <Button
                  :disabled="submittingRedeemBatch"
                  @click="submitRedeemCodeBatch"
                >
                  {{ submittingRedeemBatch ? '生成中...' : '生成兑换码' }}
                </Button>
              </div>

              <div
                v-if="latestGeneratedRedeemBatch"
                class="rounded-xl border border-border/60 bg-muted/20 p-3 text-xs text-muted-foreground"
              >
                最近生成批次:
                <span class="font-medium text-foreground">{{ latestGeneratedRedeemBatch.name }}</span>
                · {{ latestGeneratedRedeemCodes.length }} 个兑换码
              </div>
            </div>

            <div class="grid gap-5 xl:grid-cols-[1.1fr_1fr]">
              <div class="space-y-4">
                <div class="flex flex-wrap items-center gap-2">
                  <Select v-model="redeemBatchStatusFilter">
                    <SelectTrigger class="w-[180px]">
                      <SelectValue placeholder="批次状态" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">
                        全部状态
                      </SelectItem>
                      <SelectItem value="active">
                        可用
                      </SelectItem>
                      <SelectItem value="disabled">
                        已停用
                      </SelectItem>
                    </SelectContent>
                  </Select>
                  <div class="text-sm text-muted-foreground">
                    共 {{ redeemBatchTotal }} 个批次
                  </div>
                </div>

                <div class="rounded-2xl border border-border/60 overflow-hidden bg-background">
                  <div class="overflow-x-auto">
                    <Table class="w-full min-w-[800px] table-fixed">
                      <colgroup>
                        <col :style="{ width: redeemBatchTableColumnWidths.batch }">
                        <col :style="{ width: redeemBatchTableColumnWidths.amount }">
                        <col :style="{ width: redeemBatchTableColumnWidths.count }">
                        <col :style="{ width: redeemBatchTableColumnWidths.status }">
                        <col :style="{ width: redeemBatchTableColumnWidths.actions }">
                      </colgroup>
                      <TableHeader>
                        <TableRow>
                          <SortableTableHead :sortable="false" resize-column-key="batch" :resizable="true" @resize-start="handleRedeemBatchTableColumnResizeStart">
                            批次
                          </SortableTableHead>
                          <SortableTableHead :sortable="false" resize-column-key="amount" :resizable="true" @resize-start="handleRedeemBatchTableColumnResizeStart">
                            面额
                          </SortableTableHead>
                          <SortableTableHead :sortable="false" resize-column-key="count" :resizable="true" @resize-start="handleRedeemBatchTableColumnResizeStart">
                            数量
                          </SortableTableHead>
                          <SortableTableHead :sortable="false" resize-column-key="status" :resizable="true" @resize-start="handleRedeemBatchTableColumnResizeStart">
                            状态
                          </SortableTableHead>
                          <SortableTableHead class="text-right" :sortable="false" align="right" resize-column-key="actions" :resizable="true" @resize-start="handleRedeemBatchTableColumnResizeStart">
                            操作
                          </SortableTableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        <TableRow
                          v-for="batch in redeemBatches"
                          :key="batch.id"
                          class="hover:bg-muted/20"
                          :class="batch.id === selectedRedeemBatchId ? 'bg-muted/30 ring-1 ring-border/60' : ''"
                        >
                          <TableCell>
                            <div
                              class="break-words text-sm font-medium"
                              :title="batch.name"
                            >
                              {{ batch.name }}
                            </div>
                            <div class="text-xs text-muted-foreground mt-1">
                              过期: {{ formatDateTime(batch.expires_at) }}
                            </div>
                          </TableCell>
                          <TableCell class="tabular-nums">
                            {{ formatCurrency(batch.amount_usd) }}
                          </TableCell>
                          <TableCell class="text-xs text-muted-foreground">
                            {{ batch.redeemed_count }} / {{ batch.total_count }} 已使用
                          </TableCell>
                          <TableCell>
                            <Badge :variant="batch.status === 'active' ? 'success' : 'secondary'">
                              {{ batch.status === 'active' ? '可用' : '已停用' }}
                            </Badge>
                          </TableCell>
                          <TableCell class="text-right">
                            <div class="flex justify-end gap-2">
                              <Button
                                size="sm"
                                :variant="batch.id === selectedRedeemBatchId ? 'default' : 'outline'"
                                @click="selectRedeemBatch(batch)"
                              >
                                {{ batch.id === selectedRedeemBatchId ? '当前查看' : '查看码' }}
                              </Button>
                              <Button
                                v-if="batch.status === 'active'"
                                size="sm"
                                variant="destructive"
                                @click="disableRedeemBatch(batch.id)"
                              >
                                停用批次
                              </Button>
                              <Button
                                v-if="batch.status === 'disabled'"
                                size="sm"
                                variant="destructive"
                                :disabled="batch.redeemed_count > 0"
                                @click="deleteRedeemBatch(batch)"
                              >
                                删除批次
                              </Button>
                            </div>
                          </TableCell>
                        </TableRow>
                        <TableRow v-if="!loadingRedeemBatches && redeemBatches.length === 0">
                          <TableCell
                            colspan="5"
                            class="py-10"
                          >
                            <EmptyState
                              title="暂无兑换码批次"
                              description="创建批次后会在这里显示"
                            />
                          </TableCell>
                        </TableRow>
                      </TableBody>
                    </Table>
                  </div>
                </div>

                <Pagination
                  :current="redeemBatchPage"
                  :total="redeemBatchTotal"
                  :page-size="redeemBatchPageSize"
                  @update:current="handleRedeemBatchPageChange"
                  @update:page-size="handleRedeemBatchPageSizeChange"
                />
              </div>

              <div
                ref="redeemCodesPanelRef"
                class="space-y-4"
              >
                <div class="flex flex-wrap items-center justify-between gap-2">
                  <div>
                    <h4 class="text-sm font-semibold">
                      {{ currentRedeemBatch?.name || '兑换码列表' }}
                    </h4>
                    <p class="text-xs text-muted-foreground mt-1">
                      {{ currentRedeemBatch ? `面额 ${formatCurrency(currentRedeemBatch.amount_usd)} · 剩余 ${currentRedeemBatch.active_count}` : '先从左侧选择一个批次' }}
                    </p>
                  </div>
                  <div class="flex flex-wrap items-center gap-3">
                    <div class="flex items-center gap-2">
                      <span class="text-xs text-muted-foreground">显示明文</span>
                      <Switch
                        :model-value="showPlainRedeemCodes"
                        :disabled="!canRevealPlainRedeemCodes"
                        @update:model-value="showPlainRedeemCodes = Boolean($event)"
                      />
                    </div>
                    <Select v-model="redeemCodeStatusFilter">
                      <SelectTrigger class="w-[180px]">
                        <SelectValue placeholder="码状态" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="all">
                          全部状态
                        </SelectItem>
                        <SelectItem value="active">
                          可用
                        </SelectItem>
                        <SelectItem value="disabled">
                          已停用
                        </SelectItem>
                        <SelectItem value="redeemed">
                          已兑换
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <div class="text-xs text-muted-foreground">
                  {{
                    canRevealPlainRedeemCodes
                      ? '当前批次属于本次生成，已支持明文显示开关。'
                      : '仅当前会话内最近生成的一批兑换码支持明文显示；其余批次仅显示脱敏码。'
                  }}
                </div>

                <div class="rounded-2xl border border-border/60 overflow-hidden bg-background">
                  <div class="overflow-x-auto">
                    <Table class="w-full min-w-[900px] table-fixed">
                      <colgroup>
                        <col :style="{ width: redeemCodeTableColumnWidths.code }">
                        <col :style="{ width: redeemCodeTableColumnWidths.status }">
                        <col :style="{ width: redeemCodeTableColumnWidths.redeemer }">
                        <col :style="{ width: redeemCodeTableColumnWidths.order }">
                        <col :style="{ width: redeemCodeTableColumnWidths.actions }">
                      </colgroup>
                      <TableHeader>
                        <TableRow>
                          <SortableTableHead :sortable="false" resize-column-key="code" :resizable="true" @resize-start="handleRedeemCodeTableColumnResizeStart">
                            兑换码
                          </SortableTableHead>
                          <SortableTableHead :sortable="false" resize-column-key="status" :resizable="true" @resize-start="handleRedeemCodeTableColumnResizeStart">
                            状态
                          </SortableTableHead>
                          <SortableTableHead :sortable="false" resize-column-key="redeemer" :resizable="true" @resize-start="handleRedeemCodeTableColumnResizeStart">
                            兑换用户
                          </SortableTableHead>
                          <SortableTableHead :sortable="false" resize-column-key="order" :resizable="true" @resize-start="handleRedeemCodeTableColumnResizeStart">
                            关联订单
                          </SortableTableHead>
                          <SortableTableHead class="text-right" :sortable="false" align="right" resize-column-key="actions" :resizable="true" @resize-start="handleRedeemCodeTableColumnResizeStart">
                            操作
                          </SortableTableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        <TableRow
                          v-for="code in redeemCodes"
                          :key="code.id"
                        >
                          <TableCell
                            class="font-mono text-xs break-all"
                            :title="displayRedeemCode(code)"
                          >
                            {{ displayRedeemCode(code) }}
                          </TableCell>
                          <TableCell>
                            <Badge :variant="redeemCodeStatusBadge(code.status)">
                              {{ redeemCodeStatusLabel(code.status) }}
                            </Badge>
                          </TableCell>
                          <TableCell class="text-xs text-muted-foreground">
                            {{ code.redeemed_by_user_name || code.redeemed_by_user_id || '-' }}
                          </TableCell>
                          <TableCell
                            class="font-mono text-xs break-all"
                            :title="code.redeemed_order_no || code.redeemed_payment_order_id || '-'"
                          >
                            {{ code.redeemed_order_no || code.redeemed_payment_order_id || '-' }}
                          </TableCell>
                          <TableCell class="text-right">
                            <Button
                              v-if="code.status === 'active'"
                              size="sm"
                              variant="outline"
                              @click="disableRedeemCode(code.id)"
                            >
                              停用
                            </Button>
                          </TableCell>
                        </TableRow>
                        <TableRow v-if="!loadingRedeemCodes && redeemCodes.length === 0">
                          <TableCell
                            colspan="5"
                            class="py-10"
                          >
                            <EmptyState
                              title="暂无兑换码"
                              description="选择左侧批次后会显示兑换码明细"
                            />
                          </TableCell>
                        </TableRow>
                      </TableBody>
                    </Table>
                  </div>
                </div>

                <Pagination
                  :current="redeemCodePage"
                  :total="redeemCodeTotal"
                  :page-size="redeemCodePageSize"
                  @update:current="handleRedeemCodePageChange"
                  @update:page-size="handleRedeemCodePageSizeChange"
                />
              </div>
            </div>
          </TabsContent>
        </Tabs>
      </div>
    </Card>

    <Teleport to="body">
      <Transition name="drawer">
        <div
          v-if="showLedgerDrawer && currentLedger"
          class="fixed inset-0 z-[80] flex justify-end"
        >
          <div
            class="absolute inset-0 bg-black/35 backdrop-blur-sm"
            @click="closeLedgerDrawer"
          />
          <div class="drawer-panel relative h-full w-full sm:w-[760px] lg:w-[860px] sm:max-w-[95vw] border-l border-border bg-background shadow-2xl overflow-y-auto">
            <div class="sticky top-0 z-10 border-b border-border bg-background/95 backdrop-blur px-4 py-3 sm:px-6 sm:py-4">
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <h3 class="text-lg font-semibold text-foreground leading-tight">
                    流水详情
                  </h3>
                  <p class="text-xs text-muted-foreground">
                    资金动作审计信息
                  </p>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-9 w-9 shrink-0"
                  title="关闭"
                  @click="closeLedgerDrawer"
                >
                  <X class="h-4 w-4" />
                </Button>
              </div>
            </div>

            <div class="p-4 sm:p-6 space-y-5">
              <div class="rounded-2xl border border-border/60 bg-muted/30 p-4 space-y-3">
                <div class="flex flex-wrap items-center justify-between gap-2">
                  <div class="flex items-center gap-2">
                    <Badge variant="outline">
                      {{ walletTransactionCategoryLabel(currentLedger.category) }}
                    </Badge>
                    <Badge variant="secondary">
                      {{ walletTransactionReasonLabel(currentLedger.reason_code) }}
                    </Badge>
                  </div>
                  <span
                    class="text-sm font-semibold tabular-nums"
                    :class="currentLedger.amount >= 0 ? 'text-emerald-600' : 'text-rose-600'"
                  >
                    {{ currentLedger.amount >= 0 ? '+' : '' }}{{ currentLedger.amount.toFixed(4) }}
                  </span>
                </div>
                <div class="text-xs text-muted-foreground">
                  {{ formatDateTime(currentLedger.created_at) }}
                </div>
              </div>

              <div class="grid gap-3 sm:grid-cols-2">
                <div class="rounded-xl border border-border/60 p-3">
                  <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                    归属
                  </div>
                  <div class="mt-1 text-sm font-medium">
                    {{ ownerDisplayName(currentLedger.owner_name, currentLedger.owner_type) }}
                  </div>
                  <div class="mt-1 text-xs text-muted-foreground flex items-center gap-2">
                    <span>{{ ownerTypeLabel(currentLedger.owner_type) }}</span>
                    <Badge
                      v-if="currentLedger.wallet_status"
                      variant="outline"
                      class="text-[10px]"
                    >
                      {{ walletStatusLabel(currentLedger.wallet_status) }}
                    </Badge>
                  </div>
                </div>
                <div class="rounded-xl border border-border/60 p-3">
                  <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                    余额变化
                  </div>
                  <div class="mt-1 text-sm font-medium tabular-nums">
                    {{ currentLedger.balance_before.toFixed(4) }} → {{ currentLedger.balance_after.toFixed(4) }}
                  </div>
                  <div
                    v-if="currentLedger.recharge_balance_before !== null && currentLedger.recharge_balance_before !== undefined && currentLedger.gift_balance_before !== null && currentLedger.gift_balance_before !== undefined"
                    class="mt-1 text-xs text-muted-foreground tabular-nums"
                  >
                    充 {{ Number(currentLedger.recharge_balance_before).toFixed(4) }}→{{ Number(currentLedger.recharge_balance_after ?? 0).toFixed(4) }}
                    · 赠 {{ Number(currentLedger.gift_balance_before).toFixed(4) }}→{{ Number(currentLedger.gift_balance_after ?? 0).toFixed(4) }}
                  </div>
                </div>
              </div>

              <div class="grid gap-3 sm:grid-cols-2">
                <div class="rounded-xl border border-border/60 p-3">
                  <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                    关联类型
                  </div>
                  <div class="mt-1 text-sm font-medium break-all">
                    {{ walletLinkTypeLabel(currentLedger.link_type) }}
                  </div>
                </div>
                <div class="rounded-xl border border-border/60 p-3">
                  <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                    交易ID
                  </div>
                  <div class="mt-1 text-sm font-mono break-all">
                    {{ currentLedger.id }}
                  </div>
                </div>
              </div>

              <div
                v-if="currentLedger.link_type === 'payment_order'"
                class="grid gap-3 sm:grid-cols-2"
              >
                <div class="rounded-xl border border-border/60 p-3">
                  <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                    支付方式
                  </div>
                  <div class="mt-1 text-sm font-medium">
                    <span v-if="loadingLedgerOrderNo">加载中...</span>
                    <span v-else>{{ ledgerPaymentMethodDisplay }}</span>
                  </div>
                </div>
                <div class="rounded-xl border border-border/60 p-3">
                  <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                    支付订单号
                  </div>
                  <div class="mt-1 text-sm font-mono break-all">
                    <span v-if="loadingLedgerOrderNo">加载中...</span>
                    <span v-else>{{ ledgerPaymentOrderNo || '-' }}</span>
                  </div>
                </div>
              </div>

              <div class="rounded-xl border border-border/60 p-3">
                <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                  操作用户
                </div>
                <div class="mt-1 text-sm font-medium">
                  {{ currentLedger.operator_name || (currentLedger.operator_id ? '已删除用户' : '系统自动') }}
                </div>
                <div class="mt-1 text-xs text-muted-foreground">
                  ID: {{ currentLedger.operator_id || '-' }}
                </div>
                <div
                  v-if="currentLedger.operator_email"
                  class="mt-1 text-xs text-muted-foreground"
                >
                  邮箱: {{ currentLedger.operator_email }}
                </div>
              </div>

              <div class="rounded-xl border border-border/60 p-3">
                <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                  说明
                </div>
                <div class="mt-1 text-sm text-foreground whitespace-pre-wrap break-words">
                  {{ currentLedger.description || '-' }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <Teleport to="body">
      <Transition name="drawer">
        <div
          v-if="showRefundDrawer && currentRefund"
          class="fixed inset-0 z-[80] flex justify-end"
        >
          <div
            class="absolute inset-0 bg-black/35 backdrop-blur-sm"
            @click="closeRefundDrawer"
          />
          <div class="drawer-panel relative h-full w-full sm:w-[760px] lg:w-[860px] sm:max-w-[95vw] border-l border-border bg-background shadow-2xl overflow-y-auto">
            <div class="sticky top-0 z-10 border-b border-border bg-background/95 backdrop-blur px-4 py-3 sm:px-6 sm:py-4">
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <h3 class="text-lg font-semibold text-foreground leading-tight">
                    退款审批
                  </h3>
                  <p class="text-xs text-muted-foreground">
                    退款单: {{ currentRefund.refund_no }}
                  </p>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-9 w-9 shrink-0"
                  title="关闭"
                  @click="closeRefundDrawer"
                >
                  <X class="h-4 w-4" />
                </Button>
              </div>
            </div>

            <div class="p-4 sm:p-6 space-y-5">
              <div class="rounded-2xl border border-border/60 bg-muted/30 p-4">
                <div class="grid gap-3 sm:grid-cols-2">
                  <div>
                    <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                      归属
                    </div>
                    <div class="mt-1 text-sm font-medium">
                      {{ ownerDisplayName(currentRefund.owner_name, currentRefund.owner_type) }}
                    </div>
                    <div class="mt-1 text-xs text-muted-foreground">
                      {{ ownerTypeLabel(currentRefund.owner_type) }}
                    </div>
                  </div>
                  <div>
                    <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                      金额
                    </div>
                    <div class="mt-1 text-sm font-semibold tabular-nums">
                      {{ formatCurrency(currentRefund.amount_usd) }}
                    </div>
                  </div>
                  <div>
                    <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                      退款模式
                    </div>
                    <div class="mt-1 text-sm">
                      {{ refundModeLabel(currentRefund.refund_mode) }}
                    </div>
                  </div>
                  <div>
                    <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                      状态
                    </div>
                    <div class="mt-1">
                      <Badge :variant="refundStatusBadge(currentRefund.status)">
                        {{ refundStatusLabel(currentRefund.status) }}
                      </Badge>
                    </div>
                  </div>
                  <div class="sm:col-span-2">
                    <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                      申请原因
                    </div>
                    <div class="mt-1 text-sm text-foreground whitespace-pre-wrap break-words">
                      {{ currentRefund.reason || '-' }}
                    </div>
                  </div>
                  <div
                    v-if="currentRefund.failure_reason"
                    class="sm:col-span-2"
                  >
                    <div class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                      失败原因
                    </div>
                    <div class="mt-1 text-sm text-rose-600 whitespace-pre-wrap break-words">
                      {{ currentRefund.failure_reason }}
                    </div>
                  </div>
                </div>
              </div>

              <div
                v-if="canFailRefund(currentRefund.status)"
                class="rounded-xl border border-border/60 p-4 space-y-2"
              >
                <Label>驳回原因</Label>
                <Input
                  v-model="failRefundForm.reason"
                  placeholder="请填写驳回原因"
                />
              </div>

              <div
                v-if="canCompleteRefund(currentRefund.status)"
                class="rounded-xl border border-border/60 p-4 space-y-3"
              >
                <div class="space-y-1.5">
                  <Label>网关退款号（可选）</Label>
                  <Input v-model="completeRefundForm.gateway_refund_id" />
                </div>
                <div class="space-y-1.5">
                  <Label>打款凭证 / 参考号（可选）</Label>
                  <Input v-model="completeRefundForm.payout_reference" />
                </div>
              </div>
            </div>

            <div class="sticky bottom-0 border-t border-border bg-background/95 backdrop-blur px-4 py-3 sm:px-6 sm:py-4">
              <div class="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
                <Button
                  variant="outline"
                  @click="closeRefundDrawer"
                >
                  关闭
                </Button>
                <Button
                  v-if="canProcessRefund(currentRefund.status)"
                  variant="outline"
                  :disabled="submittingRefundAction"
                  @click="processRefund(currentRefund)"
                >
                  {{ submittingRefundAction ? '处理中...' : '处理退款' }}
                </Button>
                <Button
                  v-if="canCompleteRefund(currentRefund.status)"
                  :disabled="submittingRefundAction"
                  @click="submitCompleteRefund"
                >
                  {{ submittingRefundAction ? '提交中...' : '确认完成' }}
                </Button>
                <Button
                  v-if="canFailRefund(currentRefund.status)"
                  variant="destructive"
                  :disabled="submittingRefundAction"
                  @click="submitFailRefund"
                >
                  {{ submittingRefundAction ? '提交中...' : '驳回退款' }}
                </Button>
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <Dialog v-model="showCreditDialog">
      <template #header>
        <div class="px-6 py-4 border-b border-border">
          <h3 class="text-lg font-semibold">
            {{ currentOrder ? paymentOrderCreditDialogTitle(currentOrder) : '处理订单' }}
          </h3>
          <p class="text-xs text-muted-foreground mt-1">
            订单: {{ currentOrder?.order_no || '-' }}
          </p>
        </div>
      </template>
      <div class="space-y-4">
        <div class="space-y-1.5">
          <Label>网关订单号（可选）</Label>
          <Input v-model="creditForm.gateway_order_id" />
        </div>
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <div class="space-y-1.5">
            <Label>实付金额（可选）</Label>
            <Input
              v-model.number="creditForm.pay_amount"
              type="number"
              min="0.01"
              step="0.01"
            />
          </div>
          <div class="space-y-1.5">
            <Label>币种（可选）</Label>
            <Input v-model="creditForm.pay_currency" />
          </div>
          <div class="space-y-1.5">
            <Label>汇率（可选）</Label>
            <Input
              v-model.number="creditForm.exchange_rate"
              type="number"
              min="0.000001"
              step="0.000001"
            />
          </div>
        </div>
      </div>
      <template #footer>
        <Button
          variant="outline"
          @click="showCreditDialog = false"
        >
          取消
        </Button>
        <Button
          :disabled="submittingOrderAction"
          @click="submitCreditOrder"
        >
          {{ submittingOrderAction ? '提交中...' : (currentOrder ? paymentOrderCreditActionLabel(currentOrder) : '确认') }}
        </Button>
      </template>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import {
  Badge,
  Button,
  Card,
  Dialog,
  Input,
  Label,
  Pagination,
  RefreshButton,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableRow,
  SortableTableHead,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Textarea,
} from '@/components/ui'
import { EmptyState } from '@/components/common'
import { X } from 'lucide-vue-next'
import { useResizableTableColumns, type ResizableTableColumn } from '@/composables/useResizableTableColumns'
import {
  adminWalletApi,
  type AdminGlobalRefund,
  type AdminLedgerTransaction,
} from '@/api/admin-wallets'
import {
  adminPaymentsApi,
  type PaymentCallbackRecord,
  type RedeemCodeBatch,
  type RedeemCodeRecord,
} from '@/api/admin-payments'
import type { PaymentOrder } from '@/api/wallet'
import { parseApiError } from '@/utils/errorParser'
import { useToast } from '@/composables/useToast'
import { log } from '@/utils/logger'
import {
  callbackStatusBadge,
  callbackStatusLabel,
  formatWalletCurrency as formatCurrency,
  paymentMethodLabel,
  paymentOrderContentLabel,
  paymentOrderMethodLabel,
  paymentStatusBadge,
  paymentStatusLabel,
  refundModeLabel,
  refundStatusBadge,
  refundStatusLabel,
  walletLinkTypeLabel,
  walletStatusLabel,
  walletTransactionCategoryLabel,
  walletTransactionReasonLabel,
} from '@/utils/walletDisplay'

type WalletManagementTab = 'ledger' | 'refunds' | 'orders' | 'callbacks' | 'redeem_codes'
type LedgerCategory = 'recharge' | 'gift' | 'adjust' | 'refund'
type LedgerReasonOption = {
  value: string
  label: string
  category: LedgerCategory
}

const LEDGER_REASON_OPTIONS: LedgerReasonOption[] = [
  { value: 'topup_admin_manual', label: '人工充值', category: 'recharge' },
  { value: 'topup_gateway', label: '支付充值', category: 'recharge' },
  { value: 'topup_card_code', label: '卡密充值', category: 'recharge' },
  { value: 'gift_initial', label: '初始赠款', category: 'gift' },
  { value: 'gift_campaign', label: '活动赠款', category: 'gift' },
  { value: 'gift_expire_reclaim', label: '赠款回收', category: 'gift' },
  { value: 'adjust_admin', label: '人工调账', category: 'adjust' },
  { value: 'adjust_system', label: '系统调账', category: 'adjust' },
  { value: 'refund_out', label: '退款扣减', category: 'refund' },
  { value: 'refund_revert', label: '退款回补', category: 'refund' },
]

const { success, error: showError } = useToast()

type LedgerTableColumnKey = 'time' | 'owner' | 'type' | 'amount' | 'balance' | 'description' | 'actions'
const ledgerTableColumns: ResizableTableColumn<LedgerTableColumnKey>[] = [
  { key: 'time', width: '150px', minWidth: 140 },
  { key: 'owner', width: '210px', minWidth: 180 },
  { key: 'type', width: '130px', minWidth: 120 },
  { key: 'amount', width: '120px', minWidth: 110 },
  { key: 'balance', width: '220px', minWidth: 190 },
  { key: 'description', width: '280px', minWidth: 220 },
  { key: 'actions', width: '90px', minWidth: 84 },
]
const {
  columnWidths: ledgerTableColumnWidths,
  startResize: handleLedgerTableColumnResizeStart,
} = useResizableTableColumns<LedgerTableColumnKey>({
  storageKey: 'wallet-ledger-table-column-widths',
  columns: ledgerTableColumns,
  defaultMinWidth: 84,
})

type RefundTableColumnKey = 'owner' | 'refundNo' | 'amount' | 'mode' | 'status' | 'reason' | 'created' | 'actions'
const refundTableColumns: ResizableTableColumn<RefundTableColumnKey>[] = [
  { key: 'owner', width: '210px', minWidth: 180 },
  { key: 'refundNo', width: '190px', minWidth: 160 },
  { key: 'amount', width: '120px', minWidth: 110 },
  { key: 'mode', width: '110px', minWidth: 100 },
  { key: 'status', width: '110px', minWidth: 100 },
  { key: 'reason', width: '240px', minWidth: 190 },
  { key: 'created', width: '150px', minWidth: 140 },
  { key: 'actions', width: '150px', minWidth: 140 },
]
const {
  columnWidths: refundTableColumnWidths,
  startResize: handleRefundTableColumnResizeStart,
} = useResizableTableColumns<RefundTableColumnKey>({
  storageKey: 'wallet-refunds-table-column-widths',
  columns: refundTableColumns,
  defaultMinWidth: 84,
})

type PaymentOrderTableColumnKey = 'orderNo' | 'wallet' | 'content' | 'amount' | 'method' | 'status' | 'created' | 'actions'
const paymentOrderTableColumns: ResizableTableColumn<PaymentOrderTableColumnKey>[] = [
  { key: 'orderNo', width: '210px', minWidth: 180 },
  { key: 'wallet', width: '210px', minWidth: 180 },
  { key: 'content', width: '210px', minWidth: 170 },
  { key: 'amount', width: '120px', minWidth: 110 },
  { key: 'method', width: '120px', minWidth: 110 },
  { key: 'status', width: '120px', minWidth: 110 },
  { key: 'created', width: '150px', minWidth: 140 },
  { key: 'actions', width: '150px', minWidth: 140 },
]
const {
  columnWidths: paymentOrderTableColumnWidths,
  startResize: handlePaymentOrderTableColumnResizeStart,
} = useResizableTableColumns<PaymentOrderTableColumnKey>({
  storageKey: 'wallet-payment-orders-table-column-widths',
  columns: paymentOrderTableColumns,
  defaultMinWidth: 84,
})

type CallbackTableColumnKey = 'callbackKey' | 'orderNo' | 'method' | 'signature' | 'status' | 'time'
const callbackTableColumns: ResizableTableColumn<CallbackTableColumnKey>[] = [
  { key: 'callbackKey', width: '220px', minWidth: 180 },
  { key: 'orderNo', width: '210px', minWidth: 180 },
  { key: 'method', width: '120px', minWidth: 110 },
  { key: 'signature', width: '110px', minWidth: 100 },
  { key: 'status', width: '110px', minWidth: 100 },
  { key: 'time', width: '150px', minWidth: 140 },
]
const {
  columnWidths: callbackTableColumnWidths,
  startResize: handleCallbackTableColumnResizeStart,
} = useResizableTableColumns<CallbackTableColumnKey>({
  storageKey: 'wallet-callbacks-table-column-widths',
  columns: callbackTableColumns,
  defaultMinWidth: 84,
})

type RedeemBatchTableColumnKey = 'batch' | 'amount' | 'count' | 'status' | 'actions'
const redeemBatchTableColumns: ResizableTableColumn<RedeemBatchTableColumnKey>[] = [
  { key: 'batch', width: '260px', minWidth: 220 },
  { key: 'amount', width: '120px', minWidth: 110 },
  { key: 'count', width: '150px', minWidth: 130 },
  { key: 'status', width: '120px', minWidth: 110 },
  { key: 'actions', width: '150px', minWidth: 140 },
]
const {
  columnWidths: redeemBatchTableColumnWidths,
  startResize: handleRedeemBatchTableColumnResizeStart,
} = useResizableTableColumns<RedeemBatchTableColumnKey>({
  storageKey: 'wallet-redeem-batches-table-column-widths',
  columns: redeemBatchTableColumns,
  defaultMinWidth: 84,
})

type RedeemCodeTableColumnKey = 'code' | 'status' | 'redeemer' | 'order' | 'actions'
const redeemCodeTableColumns: ResizableTableColumn<RedeemCodeTableColumnKey>[] = [
  { key: 'code', width: '260px', minWidth: 220 },
  { key: 'status', width: '120px', minWidth: 110 },
  { key: 'redeemer', width: '180px', minWidth: 150 },
  { key: 'order', width: '220px', minWidth: 180 },
  { key: 'actions', width: '120px', minWidth: 110 },
]
const {
  columnWidths: redeemCodeTableColumnWidths,
  startResize: handleRedeemCodeTableColumnResizeStart,
} = useResizableTableColumns<RedeemCodeTableColumnKey>({
  storageKey: 'wallet-redeem-codes-table-column-widths',
  columns: redeemCodeTableColumns,
  defaultMinWidth: 84,
})
const route = useRoute()

const activeTab = ref<WalletManagementTab>('ledger')

const loadingLedger = ref(false)
const loadingRefunds = ref(false)
const loadingOrders = ref(false)
const loadingCallbacks = ref(false)
const loadingRedeemBatches = ref(false)
const loadingRedeemCodes = ref(false)
const submittingRefundAction = ref(false)
const submittingOrderAction = ref(false)
const submittingRedeemBatch = ref(false)

const ledgerItems = ref<AdminLedgerTransaction[]>([])
const ledgerTotal = ref(0)
const ledgerPage = ref(1)
const ledgerPageSize = ref(20)
const ledgerCategoryFilter = ref('all')
const ledgerReasonFilter = ref('all')
const ledgerOwnerFilter = ref('all')
const ledgerUserSearch = ref('')
const ledgerReasonOptions = computed(() => {
  if (ledgerCategoryFilter.value === 'all') {
    return LEDGER_REASON_OPTIONS
  }
  return LEDGER_REASON_OPTIONS.filter((option) => option.category === ledgerCategoryFilter.value)
})

const refundItems = ref<AdminGlobalRefund[]>([])
const refundTotal = ref(0)
const refundPage = ref(1)
const refundPageSize = ref(20)
const refundStatusFilter = ref('all')
const refundOwnerFilter = ref('all')
const refundUserSearch = ref('')

const orders = ref<PaymentOrder[]>([])
const orderTotal = ref(0)
const orderPage = ref(1)
const orderPageSize = ref(20)
const orderStatusFilter = ref('all')
const orderKindFilter = ref('all')
const orderMethodFilter = ref('all')
const orderUserSearch = ref('')

const callbacks = ref<PaymentCallbackRecord[]>([])
const callbackTotal = ref(0)
const callbackPage = ref(1)
const callbackPageSize = ref(20)
const callbackMethodFilter = ref('all')

const redeemBatches = ref<RedeemCodeBatch[]>([])
const redeemBatchTotal = ref(0)
const redeemBatchPage = ref(1)
const redeemBatchPageSize = ref(20)
const redeemBatchStatusFilter = ref('all')

const redeemCodes = ref<RedeemCodeRecord[]>([])
const redeemCodeTotal = ref(0)
const redeemCodePage = ref(1)
const redeemCodePageSize = ref(20)
const redeemCodeStatusFilter = ref('all')
const selectedRedeemBatchId = ref<string | null>(null)
const currentRedeemBatch = ref<RedeemCodeBatch | null>(null)
const latestGeneratedRedeemBatch = ref<RedeemCodeBatch | null>(null)
const latestGeneratedRedeemCodes = ref<Array<{ id: string; code: string; masked_code: string }>>([])
const showPlainRedeemCodes = ref(false)
const redeemCodesPanelRef = ref<HTMLElement | null>(null)

const redeemBatchForm = reactive({
  name: '',
  amount_usd: 10,
  total_count: 20,
  expires_at: '',
  description: '',
})

const canRevealPlainRedeemCodes = computed(
  () =>
    !!currentRedeemBatch.value &&
    currentRedeemBatch.value.id === latestGeneratedRedeemBatch.value?.id &&
    latestGeneratedRedeemCodes.value.length > 0
)

const canExportLatestGeneratedRedeemCodes = computed(
  () => !!latestGeneratedRedeemBatch.value && latestGeneratedRedeemCodes.value.length > 0
)

const walletMetaMap = ref<Record<string, { ownerName: string; ownerType: 'user' | 'api_key' }>>({})

const showLedgerDrawer = ref(false)
const showRefundDrawer = ref(false)
const currentLedger = ref<AdminLedgerTransaction | null>(null)
const currentRefund = ref<AdminGlobalRefund | null>(null)
const loadingLedgerOrderNo = ref(false)
const ledgerPaymentOrderNo = ref<string | null>(null)
const ledgerPaymentMethod = ref<string | null>(null)
const ledgerPaymentOrder = ref<PaymentOrder | null>(null)

const showCreditDialog = ref(false)
const currentOrder = ref<PaymentOrder | null>(null)

const ledgerPaymentMethodDisplay = computed(() => {
  if (ledgerPaymentOrder.value) {
    return paymentOrderMethodLabel(ledgerPaymentOrder.value)
  }
  return ledgerPaymentMethod.value ? paymentMethodLabel(ledgerPaymentMethod.value) : '-'
})

const failRefundForm = reactive({
  reason: '',
})

const completeRefundForm = reactive({
  gateway_refund_id: '',
  payout_reference: '',
})

const creditForm = reactive({
  gateway_order_id: '',
  pay_amount: undefined as number | undefined,
  pay_currency: '',
  exchange_rate: undefined as number | undefined,
})

watch([ledgerCategoryFilter, ledgerReasonFilter, ledgerOwnerFilter], () => {
  ledgerPage.value = 1
  void loadLedger()
})

let ledgerUserSearchTimer: ReturnType<typeof setTimeout> | null = null
let refundUserSearchTimer: ReturnType<typeof setTimeout> | null = null
let orderUserSearchTimer: ReturnType<typeof setTimeout> | null = null

watch(ledgerUserSearch, () => {
  ledgerPage.value = 1
  if (ledgerUserSearchTimer) {
    clearTimeout(ledgerUserSearchTimer)
  }
  ledgerUserSearchTimer = setTimeout(() => {
    ledgerUserSearchTimer = null
    void loadLedger()
  }, 300)
})

watch(ledgerCategoryFilter, () => {
  if (ledgerReasonFilter.value === 'all') {
    return
  }
  const valid = ledgerReasonOptions.value.some((option) => option.value === ledgerReasonFilter.value)
  if (!valid) {
    ledgerReasonFilter.value = 'all'
  }
})

watch([refundStatusFilter, refundOwnerFilter], () => {
  refundPage.value = 1
  void loadRefunds()
})

watch(refundUserSearch, () => {
  refundPage.value = 1
  if (refundUserSearchTimer) {
    clearTimeout(refundUserSearchTimer)
  }
  refundUserSearchTimer = setTimeout(() => {
    refundUserSearchTimer = null
    void loadRefunds()
  }, 300)
})

watch([orderStatusFilter, orderKindFilter, orderMethodFilter], () => {
  orderPage.value = 1
  void loadOrders()
})

watch(orderUserSearch, () => {
  orderPage.value = 1
  if (orderUserSearchTimer) {
    clearTimeout(orderUserSearchTimer)
  }
  orderUserSearchTimer = setTimeout(() => {
    orderUserSearchTimer = null
    void loadOrders()
  }, 300)
})

watch(callbackMethodFilter, () => {
  callbackPage.value = 1
  void loadCallbacks()
})

watch(redeemBatchStatusFilter, () => {
  redeemBatchPage.value = 1
  void loadRedeemCodeBatches()
})

watch(redeemCodeStatusFilter, () => {
  redeemCodePage.value = 1
  void loadRedeemCodes()
})

watch(canRevealPlainRedeemCodes, (enabled) => {
  if (!enabled) {
    showPlainRedeemCodes.value = false
  }
})

watch(
  () => route.query.tab,
  (tab) => {
    const tabValue = Array.isArray(tab) ? tab[0] : tab
    if (isValidTab(tabValue)) {
      activeTab.value = tabValue
    }
  },
  { immediate: true }
)

onMounted(async () => {
  await Promise.all([
    loadWalletMetaMap(),
    loadLedger(),
    loadRefunds(),
    loadOrders(),
    loadCallbacks(),
    loadRedeemCodeBatches(),
  ])
})

onBeforeUnmount(() => {
  if (ledgerUserSearchTimer) clearTimeout(ledgerUserSearchTimer)
  if (refundUserSearchTimer) clearTimeout(refundUserSearchTimer)
  if (orderUserSearchTimer) clearTimeout(orderUserSearchTimer)
})

function isValidTab(tab: unknown): tab is WalletManagementTab {
  return tab === 'ledger' || tab === 'refunds' || tab === 'orders' || tab === 'callbacks' || tab === 'redeem_codes'
}

async function loadWalletMetaMap() {
  try {
    const wallets = await adminWalletApi.listAllWallets()
    walletMetaMap.value = wallets.reduce<Record<string, { ownerName: string; ownerType: 'user' | 'api_key' }>>(
      (acc, wallet) => {
        const ownerName =
          wallet.owner_name || (wallet.owner_type === 'user' ? '未命名用户' : '未命名密钥')
        acc[wallet.id] = {
          ownerName,
          ownerType: wallet.owner_type,
        }
        return acc
      },
      {}
    )
  } catch (error) {
    log.error('加载钱包名称映射失败:', error)
  }
}

async function loadLedger() {
  loadingLedger.value = true
  try {
    const offset = (ledgerPage.value - 1) * ledgerPageSize.value
    const resp = await adminWalletApi.listLedger({
      category: ledgerCategoryFilter.value !== 'all' ? ledgerCategoryFilter.value : undefined,
      reason_code: ledgerReasonFilter.value !== 'all' ? ledgerReasonFilter.value : undefined,
      owner_type: ledgerOwnerFilter.value !== 'all' ? ledgerOwnerFilter.value : undefined,
      user_search: normalizedSearch(ledgerUserSearch.value),
      limit: ledgerPageSize.value,
      offset,
    })
    ledgerItems.value = resp.items
    ledgerTotal.value = resp.total
  } catch (error) {
    log.error('加载全局资金流水失败:', error)
    showError(parseApiError(error, '加载全局资金流水失败'))
  } finally {
    loadingLedger.value = false
  }
}

async function loadRefunds() {
  loadingRefunds.value = true
  try {
    const offset = (refundPage.value - 1) * refundPageSize.value
    const resp = await adminWalletApi.listGlobalRefunds({
      status: refundStatusFilter.value !== 'all' ? refundStatusFilter.value : undefined,
      owner_type: refundOwnerFilter.value === 'user' ? 'user' : undefined,
      user_search: normalizedSearch(refundUserSearch.value),
      limit: refundPageSize.value,
      offset,
    })
    refundItems.value = resp.items
    refundTotal.value = resp.total
    if (currentRefund.value) {
      syncCurrentRefund(currentRefund.value.id)
    }
  } catch (error) {
    log.error('加载全局退款列表失败:', error)
    showError(parseApiError(error, '加载全局退款列表失败'))
  } finally {
    loadingRefunds.value = false
  }
}

async function loadOrders() {
  loadingOrders.value = true
  try {
    const offset = (orderPage.value - 1) * orderPageSize.value
    const resp = await adminPaymentsApi.listOrders({
      status: orderStatusFilter.value !== 'all' ? orderStatusFilter.value : undefined,
      order_kind: orderKindFilter.value !== 'all' ? orderKindFilter.value : undefined,
      payment_method: orderMethodFilter.value !== 'all' ? orderMethodFilter.value : undefined,
      user_search: normalizedSearch(orderUserSearch.value),
      limit: orderPageSize.value,
      offset,
    })
    orders.value = resp.items
    orderTotal.value = resp.total
  } catch (error) {
    log.error('加载支付订单失败:', error)
    showError(parseApiError(error, '加载支付订单失败'))
  } finally {
    loadingOrders.value = false
  }
}

function normalizedSearch(value: string): string | undefined {
  const trimmed = value.trim()
  return trimmed ? trimmed : undefined
}

async function loadCallbacks() {
  loadingCallbacks.value = true
  try {
    const offset = (callbackPage.value - 1) * callbackPageSize.value
    const resp = await adminPaymentsApi.listCallbacks({
      payment_method: callbackMethodFilter.value !== 'all' ? callbackMethodFilter.value : undefined,
      limit: callbackPageSize.value,
      offset,
    })
    callbacks.value = resp.items
    callbackTotal.value = resp.total
  } catch (error) {
    log.error('加载支付回调失败:', error)
    showError(parseApiError(error, '加载支付回调失败'))
  } finally {
    loadingCallbacks.value = false
  }
}

async function loadRedeemCodeBatches() {
  loadingRedeemBatches.value = true
  try {
    const offset = (redeemBatchPage.value - 1) * redeemBatchPageSize.value
    const resp = await adminPaymentsApi.listRedeemCodeBatches({
      status: redeemBatchStatusFilter.value !== 'all' ? redeemBatchStatusFilter.value : undefined,
      limit: redeemBatchPageSize.value,
      offset,
    })
    redeemBatches.value = resp.items
    redeemBatchTotal.value = resp.total

    if (selectedRedeemBatchId.value) {
      const latest = resp.items.find(item => item.id === selectedRedeemBatchId.value)
      if (latest) {
        currentRedeemBatch.value = latest
        await loadRedeemCodes(latest.id)
      } else {
        selectedRedeemBatchId.value = null
        currentRedeemBatch.value = null
        redeemCodes.value = []
        redeemCodeTotal.value = 0
      }
    }
  } catch (error) {
    log.error('加载兑换码批次失败:', error)
    showError(parseApiError(error, '加载兑换码批次失败'))
  } finally {
    loadingRedeemBatches.value = false
  }
}

async function loadRedeemCodes(batchId = selectedRedeemBatchId.value || undefined) {
  if (!batchId) {
    redeemCodes.value = []
    redeemCodeTotal.value = 0
    return
  }
  loadingRedeemCodes.value = true
  try {
    const offset = (redeemCodePage.value - 1) * redeemCodePageSize.value
    const resp = await adminPaymentsApi.listRedeemCodes(batchId, {
      status: redeemCodeStatusFilter.value !== 'all' ? redeemCodeStatusFilter.value : undefined,
      limit: redeemCodePageSize.value,
      offset,
    })
    currentRedeemBatch.value = resp.batch
    selectedRedeemBatchId.value = resp.batch.id
    redeemCodes.value = resp.items
    redeemCodeTotal.value = resp.total
  } catch (error) {
    log.error('加载兑换码列表失败:', error)
    showError(parseApiError(error, '加载兑换码列表失败'))
  } finally {
    loadingRedeemCodes.value = false
  }
}

async function selectRedeemBatch(batch: RedeemCodeBatch) {
  currentRedeemBatch.value = batch
  selectedRedeemBatchId.value = batch.id
  redeemCodePage.value = 1
  await loadRedeemCodes(batch.id)
  await nextTick()
  redeemCodesPanelRef.value?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

function exportRedeemCodesCsv(batch: RedeemCodeBatch, codes: Array<{ id: string; code: string; masked_code: string }>) {
  const header = ['id', 'batch_name', 'code', 'masked_code']
  const rows = codes.map(code => [code.id, batch.name, code.code, code.masked_code])
  const csv = [header, ...rows]
    .map(row => row.map(cell => `"${String(cell).replaceAll('"', '""')}"`).join(','))
    .join('\n')
  const blob = new Blob([`\uFEFF${csv}`], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `redeem-codes-${batch.name}-${batch.id}.csv`
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  URL.revokeObjectURL(url)
}

async function submitRedeemCodeBatch() {
  if (!redeemBatchForm.name.trim()) {
    showError('请填写批次名称')
    return
  }
  if (!redeemBatchForm.amount_usd || redeemBatchForm.amount_usd <= 0) {
    showError('请填写有效面额')
    return
  }
  if (!redeemBatchForm.total_count || redeemBatchForm.total_count <= 0) {
    showError('请填写有效数量')
    return
  }

  submittingRedeemBatch.value = true
  try {
    const payload = {
      name: redeemBatchForm.name.trim(),
      amount_usd: redeemBatchForm.amount_usd,
      total_count: redeemBatchForm.total_count,
      expires_at: redeemBatchForm.expires_at ? new Date(redeemBatchForm.expires_at).toISOString() : undefined,
      description: redeemBatchForm.description.trim() || undefined,
    }
    const resp = await adminPaymentsApi.createRedeemCodeBatch(payload)
    latestGeneratedRedeemBatch.value = resp.batch
    latestGeneratedRedeemCodes.value = resp.codes
    showPlainRedeemCodes.value = true
    success('兑换码批次已创建')
    redeemBatchForm.name = ''
    redeemBatchForm.description = ''
    redeemBatchForm.expires_at = ''
    currentRedeemBatch.value = resp.batch
    selectedRedeemBatchId.value = resp.batch.id
    await loadRedeemCodeBatches()
    await loadRedeemCodes(resp.batch.id)
  } catch (error) {
    log.error('创建兑换码批次失败:', error)
    showError(parseApiError(error, '创建兑换码批次失败'))
  } finally {
    submittingRedeemBatch.value = false
  }
}

function exportLatestGeneratedRedeemCodes() {
  if (!latestGeneratedRedeemBatch.value || latestGeneratedRedeemCodes.value.length === 0) {
    showError('当前没有可导出的新生成兑换码')
    return
  }
  exportRedeemCodesCsv(latestGeneratedRedeemBatch.value, latestGeneratedRedeemCodes.value)
  success('CSV 已导出')
}

function displayRedeemCode(code: RedeemCodeRecord) {
  if (!showPlainRedeemCodes.value || !canRevealPlainRedeemCodes.value) {
    return code.masked_code
  }
  return latestGeneratedRedeemCodes.value.find(item => item.id === code.id)?.code || code.masked_code
}

async function disableRedeemBatch(batchId: string) {
  try {
    await adminPaymentsApi.disableRedeemCodeBatch(batchId)
    success('批次已停用')
    await loadRedeemCodeBatches()
  } catch (error) {
    log.error('停用兑换码批次失败:', error)
    showError(parseApiError(error, '停用兑换码批次失败'))
  }
}

async function deleteRedeemBatch(batch: RedeemCodeBatch) {
  if (batch.redeemed_count > 0) {
    showError('已有兑换记录的批次不能删除')
    return
  }
  if (!window.confirm(`确认删除批次「${batch.name}」吗？删除后无法恢复。`)) {
    return
  }

  try {
    await adminPaymentsApi.deleteRedeemCodeBatch(batch.id)
    success('批次已删除')
    if (selectedRedeemBatchId.value === batch.id) {
      selectedRedeemBatchId.value = null
      currentRedeemBatch.value = null
      redeemCodes.value = []
      redeemCodeTotal.value = 0
      showPlainRedeemCodes.value = false
    }
    if (latestGeneratedRedeemBatch.value?.id === batch.id) {
      latestGeneratedRedeemBatch.value = null
      latestGeneratedRedeemCodes.value = []
      showPlainRedeemCodes.value = false
    }
    await loadRedeemCodeBatches()
  } catch (error) {
    log.error('删除兑换码批次失败:', error)
    showError(parseApiError(error, '删除兑换码批次失败'))
  }
}

async function disableRedeemCode(codeId: string) {
  try {
    await adminPaymentsApi.disableRedeemCode(codeId)
    success('兑换码已停用')
    await Promise.all([loadRedeemCodes(), loadRedeemCodeBatches()])
  } catch (error) {
    log.error('停用兑换码失败:', error)
    showError(parseApiError(error, '停用兑换码失败'))
  }
}

function orderWalletName(walletId: string) {
  return walletMetaMap.value[walletId]?.ownerName || '未知钱包'
}

function orderWalletTypeLabel(walletId: string) {
  const ownerType = walletMetaMap.value[walletId]?.ownerType
  if (!ownerType) return '未知归属'
  return ownerType === 'user' ? '用户钱包' : '独立密钥钱包'
}

function openLedgerDrawer(tx: AdminLedgerTransaction) {
  currentLedger.value = tx
  ledgerPaymentOrderNo.value = null
  ledgerPaymentMethod.value = null
  ledgerPaymentOrder.value = null
  showLedgerDrawer.value = true
  void resolveLedgerRechargeOrderNo(tx)
}

async function resolveLedgerRechargeOrderNo(tx: AdminLedgerTransaction) {
  if (tx.link_type !== 'payment_order' || !tx.link_id) {
    ledgerPaymentOrderNo.value = null
    ledgerPaymentMethod.value = null
    ledgerPaymentOrder.value = null
    return
  }

  if (tx.link_id.startsWith('po_')) {
    ledgerPaymentOrderNo.value = tx.link_id
    ledgerPaymentMethod.value = null
    ledgerPaymentOrder.value = null
    return
  }

  loadingLedgerOrderNo.value = true
  try {
    const resp = await adminPaymentsApi.getOrder(tx.link_id)
    ledgerPaymentOrder.value = resp.order
    ledgerPaymentOrderNo.value = resp.order.order_no || null
    ledgerPaymentMethod.value = resp.order.payment_method || null
  } catch (error) {
    log.error('加载关联支付订单失败:', error)
    ledgerPaymentOrder.value = null
    ledgerPaymentOrderNo.value = null
    ledgerPaymentMethod.value = null
  } finally {
    loadingLedgerOrderNo.value = false
  }
}

function closeLedgerDrawer() {
  showLedgerDrawer.value = false
}

function openRefundDrawer(refund: AdminGlobalRefund) {
  currentRefund.value = refund
  failRefundForm.reason = ''
  completeRefundForm.gateway_refund_id = ''
  completeRefundForm.payout_reference = ''
  showRefundDrawer.value = true
}

function closeRefundDrawer() {
  showRefundDrawer.value = false
}

function syncCurrentRefund(refundId: string) {
  const latest = refundItems.value.find((item) => item.id === refundId)
  if (latest) {
    currentRefund.value = latest
  }
}

async function processRefund(refund: AdminGlobalRefund) {
  submittingRefundAction.value = true
  try {
    await adminWalletApi.processRefund(refund.wallet_id, refund.id)
    success('退款已进入 processing')
    await Promise.all([loadRefunds(), loadLedger()])
    syncCurrentRefund(refund.id)
  } catch (error) {
    log.error('处理退款失败:', error)
    showError(parseApiError(error, '处理退款失败'))
  } finally {
    submittingRefundAction.value = false
  }
}

async function submitFailRefund() {
  if (!currentRefund.value) return
  if (!failRefundForm.reason.trim()) {
    showError('请填写驳回原因')
    return
  }

  submittingRefundAction.value = true
  try {
    await adminWalletApi.failRefund(currentRefund.value.wallet_id, currentRefund.value.id, {
      reason: failRefundForm.reason.trim(),
    })
    success('退款已驳回')
    await Promise.all([loadRefunds(), loadLedger()])
    syncCurrentRefund(currentRefund.value.id)
  } catch (error) {
    log.error('驳回退款失败:', error)
    showError(parseApiError(error, '驳回退款失败'))
  } finally {
    submittingRefundAction.value = false
  }
}

async function submitCompleteRefund() {
  if (!currentRefund.value) return

  submittingRefundAction.value = true
  try {
    await adminWalletApi.completeRefund(currentRefund.value.wallet_id, currentRefund.value.id, {
      gateway_refund_id: completeRefundForm.gateway_refund_id || undefined,
      payout_reference: completeRefundForm.payout_reference || undefined,
    })
    success('退款已完成')
    await Promise.all([loadRefunds(), loadLedger()])
    syncCurrentRefund(currentRefund.value.id)
  } catch (error) {
    log.error('完成退款失败:', error)
    showError(parseApiError(error, '完成退款失败'))
  } finally {
    submittingRefundAction.value = false
  }
}

function openCreditDialog(order: PaymentOrder) {
  currentOrder.value = order
  creditForm.gateway_order_id = order.gateway_order_id || ''
  creditForm.pay_amount = order.pay_amount || undefined
  creditForm.pay_currency = order.pay_currency || ''
  creditForm.exchange_rate = order.exchange_rate || undefined
  showCreditDialog.value = true
}

async function submitCreditOrder() {
  if (!currentOrder.value) return
  submittingOrderAction.value = true
  try {
    const actionLabel = paymentOrderCreditActionLabel(currentOrder.value)
    await adminPaymentsApi.creditOrder(currentOrder.value.id, {
      gateway_order_id: creditForm.gateway_order_id || undefined,
      pay_amount: creditForm.pay_amount,
      pay_currency: creditForm.pay_currency || undefined,
      exchange_rate: creditForm.exchange_rate,
    })
    success(`订单已${actionLabel.replace('确认', '')}`)
    showCreditDialog.value = false
    await Promise.all([loadOrders(), loadLedger(), loadWalletMetaMap()])
  } catch (error) {
    const actionLabel = paymentOrderCreditActionLabel(currentOrder.value)
    log.error(`${actionLabel}失败:`, error)
    showError(parseApiError(error, `${actionLabel}失败`))
  } finally {
    submittingOrderAction.value = false
  }
}

async function expireOrder(orderId: string) {
  submittingOrderAction.value = true
  try {
    await adminPaymentsApi.expireOrder(orderId)
    success('订单已标记过期')
    await loadOrders()
  } catch (error) {
    log.error('标记过期失败:', error)
    showError(parseApiError(error, '标记过期失败'))
  } finally {
    submittingOrderAction.value = false
  }
}

async function failOrder(orderId: string) {
  submittingOrderAction.value = true
  try {
    await adminPaymentsApi.failOrder(orderId)
    success('订单已标记失败')
    await loadOrders()
  } catch (error) {
    log.error('标记失败失败:', error)
    showError(parseApiError(error, '标记失败失败'))
  } finally {
    submittingOrderAction.value = false
  }
}

function canProcessRefund(status: string) {
  return status === 'pending_approval' || status === 'approved'
}

function canFailRefund(status: string) {
  return status === 'processing' || status === 'pending_approval' || status === 'approved'
}

function canCompleteRefund(status: string) {
  return status === 'processing'
}

function canCreditOrder(status: string) {
  return status === 'pending' || status === 'paid'
}

function paymentOrderCreditActionLabel(order: PaymentOrder) {
  return order.order_kind === 'plan_purchase' ? '确认发放' : '确认到账'
}

function paymentOrderCreditDialogTitle(order: PaymentOrder) {
  return order.order_kind === 'plan_purchase' ? '人工发放' : '人工到账'
}

function paymentOrderFulfillmentLabel(status: string | null | undefined) {
  const labels: Record<string, string> = {
    pending: '待发放',
    fulfilled: '已发放',
    failed: '发放失败',
  }
  if (!status) return ''
  return labels[status] || status
}

function canExpireOrder(status: string) {
  return status === 'pending'
}

function canFailOrder(status: string) {
  return status !== 'credited' && status !== 'refunded'
}

function handleLedgerPageChange(page: number) {
  ledgerPage.value = page
  void loadLedger()
}

function handleLedgerPageSizeChange(size: number) {
  ledgerPageSize.value = size
  ledgerPage.value = 1
  void loadLedger()
}

function handleRefundPageChange(page: number) {
  refundPage.value = page
  void loadRefunds()
}

function handleRefundPageSizeChange(size: number) {
  refundPageSize.value = size
  refundPage.value = 1
  void loadRefunds()
}

function handleOrderPageChange(page: number) {
  orderPage.value = page
  void loadOrders()
}

function handleOrderPageSizeChange(size: number) {
  orderPageSize.value = size
  orderPage.value = 1
  void loadOrders()
}

function handleCallbackPageChange(page: number) {
  callbackPage.value = page
  void loadCallbacks()
}

function handleCallbackPageSizeChange(size: number) {
  callbackPageSize.value = size
  callbackPage.value = 1
  void loadCallbacks()
}

function handleRedeemBatchPageChange(page: number) {
  redeemBatchPage.value = page
  void loadRedeemCodeBatches()
}

function handleRedeemBatchPageSizeChange(size: number) {
  redeemBatchPageSize.value = size
  redeemBatchPage.value = 1
  void loadRedeemCodeBatches()
}

function handleRedeemCodePageChange(page: number) {
  redeemCodePage.value = page
  void loadRedeemCodes()
}

function handleRedeemCodePageSizeChange(size: number) {
  redeemCodePageSize.value = size
  redeemCodePage.value = 1
  void loadRedeemCodes()
}

function ownerTypeLabel(ownerType: 'user' | 'api_key') {
  return ownerType === 'user' ? '用户钱包' : '独立密钥'
}

function ownerDisplayName(name: string | null | undefined, ownerType: 'user' | 'api_key') {
  if (name) return name
  return ownerType === 'user' ? '未命名用户' : '未命名密钥'
}

function formatDateTime(value: string | null | undefined) {
  if (!value) return '-'
  return new Date(value).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function redeemCodeStatusLabel(status: string) {
  if (status === 'active') return '可用'
  if (status === 'disabled') return '已停用'
  if (status === 'redeemed') return '已兑换'
  return status
}

function redeemCodeStatusBadge(status: string) {
  if (status === 'active') return 'success'
  if (status === 'disabled') return 'secondary'
  if (status === 'redeemed') return 'outline'
  return 'secondary'
}
</script>

<style scoped>
.drawer-enter-active,
.drawer-leave-active {
  transition: opacity 0.3s ease;
}

.drawer-enter-active .drawer-panel,
.drawer-leave-active .drawer-panel {
  transition: transform 0.3s ease;
}

.drawer-enter-from,
.drawer-leave-to {
  opacity: 0;
}

.drawer-enter-from .drawer-panel,
.drawer-leave-to .drawer-panel {
  transform: translateX(100%);
}
</style>
