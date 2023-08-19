CREATE TABLE Todos (
  id CHAR(24) PRIMARY KEY NOT NULL, 
  content TEXT NOT NULL,
  created_at BIGINT NOT NULL,
  completed_at BIGINT
);

CREATE INDEX Todos_created_at
ON Todos (created_at);

CREATE INDEX Todos_completed_at
ON Todos (completed_at);
