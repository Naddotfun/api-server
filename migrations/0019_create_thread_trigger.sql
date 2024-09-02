CREATE OR REPLACE FUNCTION notify_thread_changes()
RETURNS trigger AS $$
BEGIN
    PERFORM pg_notify('thread_change', json_build_object(
        'operation', TG_OP,
        'token_id', NEW.token_id,
        'record', row_to_json(NEW)
    )::text);

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- 트리거 설정
CREATE TRIGGER replicated_thread_changes_trigger
AFTER INSERT OR UPDATE ON thread
FOR EACH ROW EXECUTE FUNCTION notify_thread_changes();

-- 복제 트리거 활성화
ALTER TABLE thread ENABLE REPLICA TRIGGER replicated_thread_changes_trigger;