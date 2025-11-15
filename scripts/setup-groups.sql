-- Create groups
INSERT OR IGNORE INTO groups (name, display_order, created_at) VALUES ('Emacs', 0, datetime('now'));
INSERT OR IGNORE INTO groups (name, display_order, created_at) VALUES ('Games', 1, datetime('now'));
INSERT OR IGNORE INTO groups (name, display_order, created_at) VALUES ('CLI Tools', 2, datetime('now'));
INSERT OR IGNORE INTO groups (name, display_order, created_at) VALUES ('Web Apps', 3, datetime('now'));
INSERT OR IGNORE INTO groups (name, display_order, created_at) VALUES ('Learning/Demos', 4, datetime('now'));

-- Add emacs repos to Emacs group
INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/emacs-agent', id, datetime('now') FROM groups WHERE name = 'Emacs';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/emacs-ai-study-group', id, datetime('now') FROM groups WHERE name = 'Emacs';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/emacs-ai-api', id, datetime('now') FROM groups WHERE name = 'Emacs';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/emacs-rust-menus', id, datetime('now') FROM groups WHERE name = 'Emacs';

-- Add game repos to Games group
INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/horserace', id, datetime('now') FROM groups WHERE name = 'Games';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/one-armed', id, datetime('now') FROM groups WHERE name = 'Games';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/shut_the_box', id, datetime('now') FROM groups WHERE name = 'Games';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/game-lib', id, datetime('now') FROM groups WHERE name = 'Games';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/slots-roulette', id, datetime('now') FROM groups WHERE name = 'Games';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/ARPB', id, datetime('now') FROM groups WHERE name = 'Games';

-- Add CLI tools to CLI Tools group
INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/overall', id, datetime('now') FROM groups WHERE name = 'CLI Tools';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/proact', id, datetime('now') FROM groups WHERE name = 'CLI Tools';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/assist', id, datetime('now') FROM groups WHERE name = 'CLI Tools';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/ask', id, datetime('now') FROM groups WHERE name = 'CLI Tools';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/markdown-checker', id, datetime('now') FROM groups WHERE name = 'CLI Tools';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/needs-attention', id, datetime('now') FROM groups WHERE name = 'CLI Tools';

-- Add learning/demo repos to Learning/Demos group
INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/rag-demo', id, datetime('now') FROM groups WHERE name = 'Learning/Demos';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/neural-network-examples-rs', id, datetime('now') FROM groups WHERE name = 'Learning/Demos';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/demo-ibm-1130-system', id, datetime('now') FROM groups WHERE name = 'Learning/Demos';

INSERT OR IGNORE INTO repo_groups (repo_id, group_id, added_at)
SELECT 'softwarewrighter/logic-effects', id, datetime('now') FROM groups WHERE name = 'Learning/Demos';
