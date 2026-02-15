-- Contractors
CREATE TABLE contractors (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
  bio TEXT,
  created_at TIMESTAMPTZ DEFAULT now()
);



-- Service Categories
CREATE TABLE service_categories (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
 name VARCHAR(100) NOT NULL UNIQUE,
 description TEXT,
 created_at TIMESTAMPTZ DEFAULT now()
);



-- Contractor <-> Categories (Many-to-Many)
CREATE TABLE contractor_categories (
  contractor_id UUID NOT NULL REFERENCES contractors(id) ON DELETE CASCADE,
  category_id UUID NOT NULL REFERENCES service_categories(id) ON DELETE CASCADE,
  is_available BOOLEAN DEFAULT true, -- <- As we can tamperaly pause specific stuff for new request not old, (liek sometime due to not having enugh material to do that specific work)
  is_verified BOOLEAN DEFAULT false,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL
  PRIMARY KEY (contractor_id, category_id)
);

CREATE INDEX idx_contractor_categories_category
    ON contractor_categories(category_id);

CREATE INDEX idx_contractor_categories_available
    ON contractor_categories(is_available);

CREATE UNIQUE INDEX uniq_active_contractor_category
    ON contractor_categories(contractor_id, category_id)
    WHERE deleted_at IS NULL;


CREATE TABLE order_types (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name VARCHAR(50) UNIQUE NOT NULL
);

CREATE TABLE vendor_types (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name VARCHAR(50) UNIQUE NOT NULL
);
