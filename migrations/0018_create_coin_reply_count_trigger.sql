CREATE OR REPLACE FUNCTION notify_new_coin_reply()
RETURNS trigger AS $$
BEGIN
   PERFORM pg_notify('new_coin_reply', json_build_object(
        'coin_id', NEW.coin_id,
        'record',row_to_json(NEW)
    )::text);

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 트리거 설정
CREATE TRIGGER replicated_new_coin_reply_trigger
AFTER INSERT ON coin_reply_count
FOR EACH ROW EXECUTE FUNCTION notify_new_coin_reply();

-- 복제 트리거 활성화
ALTER TABLE coin_reply_count ENABLE REPLICA TRIGGER replicated_new_coin_reply_trigger;