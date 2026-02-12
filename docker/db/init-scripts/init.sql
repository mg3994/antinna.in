-- Enable extensions
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS postgis_topology;

-- Create a test table
CREATE TABLE IF NOT EXISTS test_data (
                                         id SERIAL PRIMARY KEY,
                                         name VARCHAR(100),
    embedding vector(3),
    location geometry(POINT, 4326),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );

-- Insert sample data
INSERT INTO test_data (name, embedding, location) VALUES
                                                      ('Sample Point 1', '[1,2,3]', ST_GeomFromText('POINT(-74.006 40.7128)', 4326)),
                                                      ('Sample Point 2', '[4,5,6]', ST_GeomFromText('POINT(-73.935 40.7282)', 4326));