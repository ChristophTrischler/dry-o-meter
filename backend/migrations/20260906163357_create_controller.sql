CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE controllers (
    controller_id      SERIAL PRIMARY KEY,
    uuid               UUID NOT NULL DEFAULT gen_random_uuid(),
    name               VARCHAR(100) NOT NULL,
    min_humidity       NUMERIC(5,2),
    max_humidity       NUMERIC(5,2),
    location           VARCHAR(100) NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE humidity_readings (
    time            TIMESTAMPTZ NOT NULL,
    controller_id   INTEGER NOT NULL REFERENCES controllers(controller_id),
    humidity        NUMERIC(5,2) NOT NULL,
    battery_voltage NUMERIC(4,2),
    rssi            SMALLINT,
    is_alert        BOOLEAN NOT NULL DEFAULT FALSE
) WITH (
    tsdb.hypertable
);

CREATE INDEX idx_readings_controller_time ON humidity_readings (controller_id, time DESC);
CREATE INDEX idx_readings_alerts ON humidity_readings (time DESC) WHERE is_alert = TRUE;

-- Add unique constraint on UUID
ALTER TABLE controllers ADD CONSTRAINT controllers_uuid_key UNIQUE (uuid);

-- Add retention policy
SELECT add_retention_policy('humidity_readings', INTERVAL '1 year');
