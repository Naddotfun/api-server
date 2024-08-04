CREATE OR REPLACE FUNCTION notify_change_balance()
RETURNS trigger AS $$
BEGIN
    PERFORM pg_notify('balance_change', json_build_object(
        'operation', TG_OP,
        'coin_id', NEW.coin_id,
        'record', row_to_json(NEW)
    )::text);

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- 트리거 설정
CREATE TRIGGER replicated_change_balance_trigger
AFTER INSERT OR UPDATE OR DELETE ON balance
FOR EACH ROW EXECUTE FUNCTION notify_change_balance();

-- 복제 트리거 활성화
ALTER TABLE balance ENABLE REPLICA TRIGGER replicated_change_balance_trigger;