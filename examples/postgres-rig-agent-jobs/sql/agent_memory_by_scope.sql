select
  agent_memory_records.id,
  agent_memory_records.run_id,
  agent_memory_records.memory_kind,
  agent_memory_records.source,
  agent_memory_records.confidence,
  agent_memory_records.memory_horizon,
  agent_memory_records.retention_policy,
  agent_memory_records.embedding_ref is not null as has_embedding_ref,
  agent_memory_records.created_at,
  agent_memory_records.last_used_at
from agent_memory_records
where agent_memory_records.memory_scope = :'memory_scope'
order by agent_memory_records.created_at desc, agent_memory_records.id desc
limit 100;
