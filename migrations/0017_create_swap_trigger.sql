CREATE OR REPLACE FUNCTION notify_new_swap()
RETURNS trigger AS $$
BEGIN
    PERFORM pg_notify('new_swap', json_build_object(
        'token_id', NEW.token_id,
        'record', row_to_json(NEW)
    )::text);

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- 트리거 설정
CREATE TRIGGER replicated_new_swap_trigger
AFTER INSERT ON swap
FOR EACH ROW EXECUTE FUNCTION notify_new_swap();

-- 복제 트리거 활성화
ALTER TABLE swap ENABLE REPLICA TRIGGER replicated_new_swap_trigger;