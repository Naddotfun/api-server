CREATE OR REPLACE FUNCTION notify_new_token_reply()
RETURNS trigger AS $$
BEGIN
   PERFORM pg_notify('new_token_reply', json_build_object(
        'token_id', NEW.token_id,
        'record',row_to_json(NEW)
    )::text);

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 트리거 설정
CREATE TRIGGER replicated_new_token_reply_trigger
AFTER INSERT OR UPDATE ON token_reply_count
FOR EACH ROW EXECUTE FUNCTION notify_new_token_reply();

-- 복제 트리거 활성화
ALTER TABLE token_reply_count ENABLE REPLICA TRIGGER replicated_new_token_reply_trigger;