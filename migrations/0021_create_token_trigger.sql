CREATE OR REPLACE FUNCTION notify_new_token()
RETURNS trigger AS $$
BEGIN
    PERFORM pg_notify('new_token', json_build_object(
        'record', row_to_json(NEW),
        'token_id', NEW.id
    )::text);

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- 트리거 설정
CREATE TRIGGER replicated_new_token_trigger
AFTER UPDATE ON token
FOR EACH ROW EXECUTE FUNCTION notify_new_token();

-- 복제 트리거 활성화
ALTER TABLE token ENABLE REPLICA TRIGGER replicated_new_token_trigger;