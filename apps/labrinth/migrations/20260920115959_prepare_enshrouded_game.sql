-- Preserve the original migration checksum for databases that have already applied it.
CREATE FUNCTION sqlx_20260920120000_prepare_enshrouded_game()
RETURNS trigger AS $$
BEGIN
	IF NEW.name = 'enshrouded' AND NEW.slug IS NULL THEN
		IF EXISTS (SELECT 1 FROM games WHERE name = NEW.name) THEN
			RETURN NULL;
		END IF;
		NEW.slug := 'enshrouded';
	END IF;
	RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER sqlx_20260920120000_prepare_enshrouded_game
BEFORE INSERT ON games
FOR EACH ROW
EXECUTE FUNCTION sqlx_20260920120000_prepare_enshrouded_game();
