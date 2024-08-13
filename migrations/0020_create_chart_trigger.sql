-- 차트 테이블들을 위한 트리거 함수 생성
CREATE OR REPLACE FUNCTION notify_new_chart()
RETURNS trigger AS $$
DECLARE
    chart_type TEXT;
BEGIN
    -- 테이블 이름에서 차트 타입 추출 (예: chart_1m -> 1m)
    chart_type := substring(TG_TABLE_NAME from 7);
    
    PERFORM pg_notify('new_chart', json_build_object(
        'chart_type', chart_type,
        'coin_id', NEW.coin_id,
        'record', row_to_json(NEW)
    )::text);
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
-- 트리거 설정
-- 트리거 설정
CREATE TRIGGER chart_1m_changes_trigger
AFTER INSERT OR UPDATE ON chart_1m
FOR EACH ROW EXECUTE FUNCTION notify_new_chart();

CREATE TRIGGER chart_5m_changes_trigger
AFTER INSERT OR UPDATE ON chart_5m
FOR EACH ROW EXECUTE FUNCTION notify_new_chart();

CREATE TRIGGER chart_15m_changes_trigger
AFTER INSERT OR UPDATE ON chart_15m
FOR EACH ROW EXECUTE FUNCTION notify_new_chart();

CREATE TRIGGER chart_30m_changes_trigger
AFTER INSERT OR UPDATE ON chart_30m
FOR EACH ROW EXECUTE FUNCTION notify_new_chart();

CREATE TRIGGER chart_1h_changes_trigger
AFTER INSERT OR UPDATE ON chart_1h
FOR EACH ROW EXECUTE FUNCTION notify_new_chart();

CREATE TRIGGER chart_4h_changes_trigger
AFTER INSERT OR UPDATE ON chart_4h
FOR EACH ROW EXECUTE FUNCTION notify_new_chart();

CREATE TRIGGER chart_1d_changes_trigger
AFTER INSERT OR UPDATE ON chart_1d
FOR EACH ROW EXECUTE FUNCTION notify_new_chart();

-- 복제 트리거로 설정
ALTER TABLE chart_1m ENABLE REPLICA TRIGGER chart_1m_changes_trigger;
ALTER TABLE chart_5m ENABLE REPLICA TRIGGER chart_5m_changes_trigger;
ALTER TABLE chart_15m ENABLE REPLICA TRIGGER chart_15m_changes_trigger;
ALTER TABLE chart_30m ENABLE REPLICA TRIGGER chart_30m_changes_trigger;
ALTER TABLE chart_1h ENABLE REPLICA TRIGGER chart_1h_changes_trigger;
ALTER TABLE chart_4h ENABLE REPLICA TRIGGER chart_4h_changes_trigger;
ALTER TABLE chart_1d ENABLE REPLICA TRIGGER chart_1d_changes_trigger;