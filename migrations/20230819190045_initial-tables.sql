CREATE TABLE Todos (
  id CHAR(24) PRIMARY KEY NOT NULL, 
  content TEXT NOT NULL,
  created_at BIGINT NOT NULL,
  completed_at BIGINT
);

CREATE INDEX Todos_created_at_completed_at
ON Todos (created_at, completed_at);
