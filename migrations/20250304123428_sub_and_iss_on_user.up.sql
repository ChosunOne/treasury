ALTER TABLE "user"
ADD iss VARCHAR(255) NOT NULL,
ADD sub VARCHAR(255) NOT NULL,
ADD CONSTRAINT uq_user_iss_sub UNIQUE (iss, sub);
