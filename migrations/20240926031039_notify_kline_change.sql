-- Add migration script here
-- create function notify_kline_change()
CREATE OR REPLACE FUNCTION notify_kline_change()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify('kline_change', json_build_object('id', NEW.id, 'exchange', NEW.exchange, 'symbol', NEW.symbol, 'interval', NEW.interval, 'open_time', NEW.open_time, 'open_price', NEW.open_price, 'high_price', NEW.high_price, 'low_price', NEW.low_price, 'close_price', NEW.close_price, 'volume', NEW.volume)::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- create trigger notify_kline_change_trigger after insert on klines
CREATE TRIGGER notify_kline_change_trigger_insert
AFTER INSERT ON klines
FOR EACH ROW
EXECUTE PROCEDURE notify_kline_change();

-- create trigger notify_kline_change_trigger after update on klines
CREATE TRIGGER notify_kline_change_trigger_update
AFTER UPDATE ON klines
FOR EACH ROW
EXECUTE PROCEDURE notify_kline_change();
