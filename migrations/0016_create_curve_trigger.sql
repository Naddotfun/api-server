CREATE OR REPLACE FUNCTION notify_update_curve()
RETURNS trigger AS $$
BEGIN
    PERFORM pg_notify('update_curve', json_build_object(
        'operation', TG_OP,
        'token_id',NEW.token_id,
        'record', row_to_json(NEW)
    )::text);

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- 트리거 설정
CREATE TRIGGER replicated_update_curve_trigger
AFTER UPDATE ON curve
FOR EACH ROW EXECUTE FUNCTION notify_update_curve();

-- 복제 트리거 활성화
ALTER TABLE curve ENABLE REPLICA TRIGGER replicated_update_curve_trigger;