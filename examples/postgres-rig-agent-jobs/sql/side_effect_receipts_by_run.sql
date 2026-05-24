select
  side_effect_receipts.id as receipt_id,
  side_effect_receipts.tool_call_id,
  tool_calls.run_id,
  tool_calls.tool_name,
  side_effect_receipts.effect_kind,
  side_effect_receipts.external_system,
  side_effect_receipts.external_correlation_id,
  side_effect_receipts.idempotency_key,
  side_effect_receipts.recorded_at
from side_effect_receipts
join tool_calls
  on tool_calls.id = side_effect_receipts.tool_call_id
where tool_calls.run_id = :'run_id'::uuid
order by side_effect_receipts.recorded_at asc;
