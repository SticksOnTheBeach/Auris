CREATE TABLE employees (
                           id          INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
                           full_name   VARCHAR(100) NOT NULL,
                           badge_hash  VARCHAR(64)  NOT NULL UNIQUE
);

CREATE TABLE scans (
                       id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
                       employee_id INT          NULL REFERENCES employees(id),
                       badge_hash  VARCHAR(64)  NOT NULL,
                       borne_id    VARCHAR(50)  NOT NULL,
                       result      TEXT         NOT NULL CHECK (result IN ('ok', 'unknown', 'already_scanned')),
                       scanned_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Un badge ne peut être pointé "ok" qu'une seule fois. C'est cet index qui rend
-- le "premier pointage" atomique, même si deux bornes scannent en même temps.
CREATE UNIQUE INDEX scans_one_ok_per_badge ON scans (badge_hash) WHERE result = 'ok';

INSERT INTO employees (full_name, badge_hash) VALUES
                                                  ('Alice Martin',  encode(sha256(convert_to('04A1B2C3', 'UTF8')), 'hex')),
                                                  ('Bob Durand',    encode(sha256(convert_to('04D4E5F6', 'UTF8')), 'hex')),
                                                  ('Chloé Bernard', encode(sha256(convert_to('04A7B8C9', 'UTF8')), 'hex'));