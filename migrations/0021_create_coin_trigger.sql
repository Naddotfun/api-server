CREATE OR REPLACE FUNCTION notify_new_coin()
RETURNS trigger AS $$
BEGIN
    PERFORM pg_notify('new_coin', json_build_object(
        'record', row_to_json(NEW),
        'coin_id', NEW.id
    )::text);

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- 트리거 설정
CREATE TRIGGER replicated_new_coin_trigger
AFTER UPDATE ON coin
FOR EACH ROW EXECUTE FUNCTION notify_new_coin();

-- 복제 트리거 활성화
ALTER TABLE coin ENABLE REPLICA TRIGGER replicated_new_coin_trigger;