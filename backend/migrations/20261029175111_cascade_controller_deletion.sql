ALTER TABLE humidity_readings
    DROP CONSTRAINT humidity_readings_controller_id_fkey,
    ADD CONSTRAINT humidity_readings_controller_id_fkey
        FOREIGN KEY (controller_id)
        REFERENCES controllers (controller_id)
        ON DELETE CASCADE;
