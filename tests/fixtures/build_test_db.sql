-- Synthetic chat.db for integration testing.
-- Mirrors the Apple iMessage schema subset used by the ETL pipeline.

CREATE TABLE handle (
    ROWID    INTEGER PRIMARY KEY,
    id       TEXT NOT NULL
);

CREATE TABLE message (
    ROWID                       INTEGER PRIMARY KEY,
    text                        TEXT,
    attributedBody              BLOB,
    handle_id                   INTEGER NOT NULL DEFAULT 0,
    is_from_me                  INTEGER NOT NULL DEFAULT 0,
    date                        INTEGER NOT NULL DEFAULT 0,
    associated_message_type     INTEGER NOT NULL DEFAULT 0,
    expressive_send_style_id    TEXT,
    thread_originator_guid      TEXT,
    balloon_bundle_id           TEXT,
    is_audio_message            INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE chat_message_join (
    chat_id    INTEGER NOT NULL,
    message_id INTEGER NOT NULL
);

CREATE TABLE chat_handle_join (
    chat_id   INTEGER NOT NULL,
    handle_id INTEGER NOT NULL
);

-- Handles: two contacts
INSERT INTO handle VALUES (1, '+14155550001');   -- Alice
INSERT INTO handle VALUES (2, '+14155550002');   -- Bob

-- Apple epoch offset: 978307200 seconds from Unix epoch
-- Timestamps stored as nanoseconds since 2001-01-01
-- 2024-01-15 12:00:00 UTC = Unix 1705320000 → Apple = (1705320000 - 978307200) * 1e9
--   = 727012800000000000
-- 2024-01-16 09:30:00 UTC = Unix 1705398600 → Apple = 727091400000000000

-- Chat 1: 1-on-1 with Alice
-- msg 1: received plain text from Alice
INSERT INTO message VALUES (1, 'Hey, how are you?', NULL, 1, 0, 727012800000000000, 0, NULL, NULL, NULL, 0);
-- msg 2: sent reply (handle_id=0 for sent)
INSERT INTO message VALUES (2, 'Doing great!', NULL, 0, 1, 727012860000000000, 0, NULL, NULL, NULL, 0);
-- msg 3: received with NULL text → attributedBody fallback
--   attributedBody encodes "Call me later" between NSString and NSDictionary markers
INSERT INTO message VALUES (3, NULL, X'4e53537472696e6743616c6c206d65206c617465724e44696374696f6e617279', 1, 0, 727012920000000000, 0, NULL, NULL, NULL, 0);
-- msg 4: reaction (Loved) on msg 1 — associated_message_type=2000
INSERT INTO message VALUES (4, 'Loved "Hey, how are you?"', NULL, 1, 0, 727012980000000000, 2000, NULL, NULL, NULL, 0);
-- msg 5: sent with Fireworks effect
INSERT INTO message VALUES (5, 'Happy new year!', NULL, 0, 1, 727091400000000000, 0, 'com.apple.MobileSMS.expressivesend.CKFireworksEffect', NULL, NULL, 0);
-- msg 6: thread reply
INSERT INTO message VALUES (6, 'I meant 8pm', NULL, 1, 0, 727091460000000000, 0, NULL, 'some-guid-abc', NULL, 0);
-- msg 7: link preview (spotify)
INSERT INTO message VALUES (7, 'Check this out https://open.spotify.com/track/abc123', NULL, 1, 0, 727091520000000000, 0, NULL, NULL, 'com.apple.messages.URLBalloonProvider', 0);
-- msg 8: audio message
INSERT INTO message VALUES (8, NULL, NULL, 1, 0, 727091580000000000, 0, NULL, NULL, NULL, 1);

-- Chat 2: group chat with Alice and Bob
-- msg 9: received from Alice in group
INSERT INTO message VALUES (9, 'Group dinner tonight?', NULL, 1, 0, 727091640000000000, 0, NULL, NULL, NULL, 0);
-- msg 10: sent in group (handle_id=0)
INSERT INTO message VALUES (10, 'Sounds good!', NULL, 0, 1, 727091700000000000, 0, NULL, NULL, NULL, 0);

-- Chat-message joins
INSERT INTO chat_message_join VALUES (1, 1);
INSERT INTO chat_message_join VALUES (1, 2);
INSERT INTO chat_message_join VALUES (1, 3);
INSERT INTO chat_message_join VALUES (1, 4);
INSERT INTO chat_message_join VALUES (1, 5);
INSERT INTO chat_message_join VALUES (1, 6);
INSERT INTO chat_message_join VALUES (1, 7);
INSERT INTO chat_message_join VALUES (1, 8);
INSERT INTO chat_message_join VALUES (2, 9);
INSERT INTO chat_message_join VALUES (2, 10);

-- Chat-handle joins
-- Chat 1: just Alice
INSERT INTO chat_handle_join VALUES (1, 1);
-- Chat 2: Alice and Bob
INSERT INTO chat_handle_join VALUES (2, 1);
INSERT INTO chat_handle_join VALUES (2, 2);
