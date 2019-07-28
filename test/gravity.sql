PRAGMA FOREIGN_KEYS = ON;

-- BEGIN SCHEMA

CREATE TABLE "group"
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    enabled     BOOLEAN NOT NULL DEFAULT 1,
    name        TEXT    NOT NULL,
    description TEXT
);

CREATE TABLE whitelist
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    domain        TEXT UNIQUE NOT NULL,
    enabled       BOOLEAN     NOT NULL DEFAULT 1,
    date_added    INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    date_modified INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    comment       TEXT
);

CREATE TABLE whitelist_by_group
(
    whitelist_id INTEGER NOT NULL REFERENCES whitelist (id),
    group_id     INTEGER NOT NULL REFERENCES "group" (id),
    PRIMARY KEY (whitelist_id, group_id)
);

CREATE TABLE blacklist
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    domain        TEXT UNIQUE NOT NULL,
    enabled       BOOLEAN     NOT NULL DEFAULT 1,
    date_added    INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    date_modified INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    comment       TEXT
);

CREATE TABLE blacklist_by_group
(
    blacklist_id INTEGER NOT NULL REFERENCES blacklist (id),
    group_id     INTEGER NOT NULL REFERENCES "group" (id),
    PRIMARY KEY (blacklist_id, group_id)
);

CREATE TABLE regex
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    domain        TEXT UNIQUE NOT NULL,
    enabled       BOOLEAN     NOT NULL DEFAULT 1,
    date_added    INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    date_modified INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    comment       TEXT
);

CREATE TABLE regex_by_group
(
    regex_id INTEGER NOT NULL REFERENCES regex (id),
    group_id INTEGER NOT NULL REFERENCES "group" (id),
    PRIMARY KEY (regex_id, group_id)
);

CREATE TABLE adlist
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    address       TEXT UNIQUE NOT NULL,
    enabled       BOOLEAN     NOT NULL DEFAULT 1,
    date_added    INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    date_modified INTEGER     NOT NULL DEFAULT (cast(strftime('%s', 'now') as int)),
    comment       TEXT
);

CREATE TABLE adlist_by_group
(
    adlist_id INTEGER NOT NULL REFERENCES adlist (id),
    group_id  INTEGER NOT NULL REFERENCES "group" (id),
    PRIMARY KEY (adlist_id, group_id)
);

CREATE TABLE gravity
(
    domain TEXT PRIMARY KEY
);

CREATE TABLE info
(
    property TEXT PRIMARY KEY,
    value    TEXT NOT NULL
);

INSERT INTO info
VALUES ('version', '1');

CREATE VIEW vw_gravity AS
SELECT domain
FROM gravity
WHERE domain NOT IN (SELECT domain from vw_whitelist);

CREATE VIEW vw_whitelist AS
SELECT DISTINCT domain
FROM whitelist
         LEFT JOIN whitelist_by_group ON whitelist_by_group.whitelist_id = whitelist.id
         LEFT JOIN "group" ON "group".id = whitelist_by_group.group_id
WHERE whitelist.enabled = 1
  AND (whitelist_by_group.group_id IS NULL OR "group".enabled = 1)
ORDER BY whitelist.id;

CREATE TRIGGER tr_whitelist_update
    AFTER UPDATE
    ON whitelist
BEGIN
    UPDATE whitelist SET date_modified = (cast(strftime('%s', 'now') as int)) WHERE domain = NEW.domain;
END;

CREATE VIEW vw_blacklist AS
SELECT DISTINCT domain
FROM blacklist
         LEFT JOIN blacklist_by_group ON blacklist_by_group.blacklist_id = blacklist.id
         LEFT JOIN "group" ON "group".id = blacklist_by_group.group_id
WHERE blacklist.enabled = 1
  AND (blacklist_by_group.group_id IS NULL OR "group".enabled = 1)
ORDER BY blacklist.id;

CREATE TRIGGER tr_blacklist_update
    AFTER UPDATE
    ON blacklist
BEGIN
    UPDATE blacklist SET date_modified = (cast(strftime('%s', 'now') as int)) WHERE domain = NEW.domain;
END;

CREATE VIEW vw_regex AS
SELECT DISTINCT domain
FROM regex
         LEFT JOIN regex_by_group ON regex_by_group.regex_id = regex.id
         LEFT JOIN "group" ON "group".id = regex_by_group.group_id
WHERE regex.enabled = 1
  AND (regex_by_group.group_id IS NULL OR "group".enabled = 1)
ORDER BY regex.id;

CREATE TRIGGER tr_regex_update
    AFTER UPDATE
    ON regex
BEGIN
    UPDATE regex SET date_modified = (cast(strftime('%s', 'now') as int)) WHERE domain = NEW.domain;
END;

CREATE VIEW vw_adlist AS
SELECT DISTINCT address
FROM adlist
         LEFT JOIN adlist_by_group ON adlist_by_group.adlist_id = adlist.id
         LEFT JOIN "group" ON "group".id = adlist_by_group.group_id
WHERE adlist.enabled = 1
  AND (adlist_by_group.group_id IS NULL OR "group".enabled = 1)
ORDER BY adlist.id;

CREATE TRIGGER tr_adlist_update
    AFTER UPDATE
    ON adlist
BEGIN
    UPDATE adlist SET date_modified = (cast(strftime('%s', 'now') as int)) WHERE address = NEW.address;
END;

-- BEGIN TEST DATA

INSERT INTO whitelist
VALUES (1, 'test.com', 1, 1557712172, 1557712172, NULL),
       (2, 'disabled-white.com', 0, 1557723854, 1557723911, NULL);

INSERT INTO blacklist
VALUES (1, 'example.com', 1, 1557712177, 1557712177, NULL),
       (2, 'disabled-black.com', 0, 1557723854, 1557723864, NULL);

INSERT INTO regex
VALUES (1, '(^|\.)example\.com$', 1, 1557712181, 1557712181, NULL),
       (2, 'disabled\-regex\.com', 0, 1557723854, 1557723872, NULL);

INSERT INTO adlist
VALUES (1, 'https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts', 1, 1557712118, 1557712118,
        'Migrated from /etc/pihole/adlists.list'),
       (2, 'https://mirror1.malwaredomains.com/files/justdomains', 1, 1557712118, 1557712118,
        'Migrated from /etc/pihole/adlists.list');

INSERT INTO gravity
VALUES ('test.com'),
       ('vqubwduhbsd.com'),
       ('vquf4tcdpt22px9l2jqqq.science'),
       ('vqwdsvjygnah.com'),
       ('vqxzysmhsvloijm12fsuswlu.download'),
       ('vr-private-kunden-de.tk'),
       ('vr-private-kundes-de.tk'),
       ('vra.outbrain.com'),
       ('vra4.com'),
       ('vriaj.com');
