CREATE VIRTUAL TABLE note_search USING fts5(url UNINDEXED, title, address, body, tokenize='trigram');
CREATE VIRTUAL TABLE session_search USING fts5(id UNINDEXED, name, tokenize='trigram');
CREATE VIRTUAL TABLE session_page_search USING fts5(session_id UNINDEXED, position UNINDEXED, title, address, tokenize='trigram');

INSERT INTO note_search(rowid,url,title,address,body) SELECT rowid,url,spark_search_fold(title),spark_search_fold(url),spark_search_fold(text) FROM page_notes;
INSERT INTO session_search(rowid,id,name) SELECT id,id,spark_search_fold(name) FROM saved_sessions;
INSERT INTO session_page_search(rowid,session_id,position,title,address) SELECT rowid,session_id,position,spark_search_fold(title),spark_search_fold(url) FROM saved_session_pages;

CREATE TRIGGER note_search_insert AFTER INSERT ON page_notes BEGIN
    INSERT INTO note_search(rowid,url,title,address,body) VALUES(new.rowid,new.url,spark_search_fold(new.title),spark_search_fold(new.url),spark_search_fold(new.text));
END;
CREATE TRIGGER note_search_update AFTER UPDATE OF url,title,text ON page_notes BEGIN
    DELETE FROM note_search WHERE rowid=old.rowid;
    INSERT INTO note_search(rowid,url,title,address,body) VALUES(new.rowid,new.url,spark_search_fold(new.title),spark_search_fold(new.url),spark_search_fold(new.text));
END;
CREATE TRIGGER note_search_delete AFTER DELETE ON page_notes BEGIN
    DELETE FROM note_search WHERE rowid=old.rowid;
END;
CREATE TRIGGER session_search_insert AFTER INSERT ON saved_sessions BEGIN
    INSERT INTO session_search(rowid,id,name) VALUES(new.id,new.id,spark_search_fold(new.name));
END;
CREATE TRIGGER session_search_update AFTER UPDATE ON saved_sessions BEGIN
    DELETE FROM session_search WHERE rowid=old.id;
    INSERT INTO session_search(rowid,id,name) VALUES(new.id,new.id,spark_search_fold(new.name));
END;
CREATE TRIGGER session_search_delete AFTER DELETE ON saved_sessions BEGIN
    DELETE FROM session_search WHERE rowid=old.id;
END;
CREATE TRIGGER session_page_search_insert AFTER INSERT ON saved_session_pages BEGIN
    INSERT INTO session_page_search(rowid,session_id,position,title,address) VALUES(new.rowid,new.session_id,new.position,spark_search_fold(new.title),spark_search_fold(new.url));
END;
CREATE TRIGGER session_page_search_update AFTER UPDATE ON saved_session_pages BEGIN
    DELETE FROM session_page_search WHERE rowid=old.rowid;
    INSERT INTO session_page_search(rowid,session_id,position,title,address) VALUES(new.rowid,new.session_id,new.position,spark_search_fold(new.title),spark_search_fold(new.url));
END;
CREATE TRIGGER session_page_search_delete AFTER DELETE ON saved_session_pages BEGIN
    DELETE FROM session_page_search WHERE rowid=old.rowid;
END;

CREATE INDEX IF NOT EXISTS idx_notes_recent ON page_notes(updated_at DESC,url);
