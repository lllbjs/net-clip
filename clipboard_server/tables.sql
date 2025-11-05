-- 用户表
CREATE TABLE `clip_users` (
                              `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT COMMENT '用户ID',
                              `username` varchar(50) NOT NULL COMMENT '用户名',
                              `email` varchar(100) NOT NULL COMMENT '邮箱',
                              `password_hash` varchar(255) NOT NULL COMMENT '密码哈希',
                              `salt` varchar(50) NOT NULL COMMENT '密码盐值',
                              `status` tinyint(1) NOT NULL DEFAULT '1' COMMENT '状态：0-禁用，1-正常',
                              `last_login_at` datetime DEFAULT NULL COMMENT '最后登录时间',
                              `last_login_ip` varchar(45) DEFAULT NULL COMMENT '最后登录IP',
                              `login_count` int(11) NOT NULL DEFAULT '0' COMMENT '登录次数',
                              `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
                              `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
                              `deleted_at` datetime DEFAULT NULL COMMENT '删除时间',
                              PRIMARY KEY (`id`),
                              UNIQUE KEY `uk_username` (`username`),
                              UNIQUE KEY `uk_email` (`email`),
                              KEY `idx_status` (`status`),
                              KEY `idx_created_at` (`created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='用户表';

-- 用户会话表（用于存储登录token）
CREATE TABLE `clip_user_sessions` (
                                      `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT COMMENT '会话ID',
                                      `user_id` bigint(20) unsigned NOT NULL COMMENT '用户ID',
                                      `token` varchar(255) NOT NULL COMMENT '访问令牌',
                                      `refresh_token` varchar(255) NOT NULL COMMENT '刷新令牌',
                                      `expires_at` datetime NOT NULL COMMENT '令牌过期时间',
                                      `refresh_expires_at` datetime NOT NULL COMMENT '刷新令牌过期时间',
                                      `device_info` varchar(500) DEFAULT NULL COMMENT '设备信息',
                                      `ip_address` varchar(45) DEFAULT NULL COMMENT 'IP地址',
                                      `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
                                      `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
                                      PRIMARY KEY (`id`),
                                      UNIQUE KEY `uk_token` (`token`),
                                      UNIQUE KEY `uk_refresh_token` (`refresh_token`),
                                      KEY `idx_user_id` (`user_id`),
                                      KEY `idx_expires_at` (`expires_at`),
                                      CONSTRAINT `fk_session_user` FOREIGN KEY (`user_id`) REFERENCES `clip_users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='用户会话表';

-- Clip内容表
CREATE TABLE `clip_contents` (
                                 `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT COMMENT '内容ID',
                                 `user_id` bigint(20) unsigned NOT NULL COMMENT '用户ID',
                                 `title` varchar(255) DEFAULT NULL COMMENT '标题',
                                 `content` text NOT NULL COMMENT '内容',
                                 `content_type` varchar(20) NOT NULL DEFAULT 'text' COMMENT '内容类型：text, code, markdown, url',
                                 `language` varchar(50) DEFAULT NULL COMMENT '编程语言（针对代码类型）',
                                 `is_encrypted` tinyint(1) NOT NULL DEFAULT '0' COMMENT '是否加密：0-否，1-是',
                                 `encryption_key` varchar(255) DEFAULT NULL COMMENT '加密密钥（加密存储）',
                                 `access_type` varchar(20) NOT NULL DEFAULT 'private' COMMENT '访问类型：private, public, unlisted',
                                 `view_count` int(11) NOT NULL DEFAULT '0' COMMENT '查看次数',
                                 `expires_at` datetime DEFAULT NULL COMMENT '过期时间',
                                 `short_url` varchar(50) DEFAULT NULL COMMENT '短链接',
                                 `tags` json DEFAULT NULL COMMENT '标签数组',
                                 `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
                                 `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
                                 `deleted_at` datetime DEFAULT NULL COMMENT '删除时间',
                                 PRIMARY KEY (`id`),
                                 UNIQUE KEY `uk_short_url` (`short_url`),
                                 KEY `idx_user_id` (`user_id`),
                                 KEY `idx_access_type` (`access_type`),
                                 KEY `idx_content_type` (`content_type`),
                                 KEY `idx_expires_at` (`expires_at`),
                                 KEY `idx_created_at` (`created_at`),
                                 FULLTEXT KEY `ft_content` (`title`, `content`),
                                 CONSTRAINT `fk_content_user` FOREIGN KEY (`user_id`) REFERENCES `clip_users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Clip内容表';

-- Clip访问日志表
CREATE TABLE `clip_access_logs` (
                                    `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT COMMENT '日志ID',
                                    `content_id` bigint(20) unsigned NOT NULL COMMENT '内容ID',
                                    `user_id` bigint(20) unsigned DEFAULT NULL COMMENT '访问用户ID（如果是登录用户）',
                                    `access_ip` varchar(45) NOT NULL COMMENT '访问IP',
                                    `user_agent` varchar(500) DEFAULT NULL COMMENT '用户代理',
                                    `referrer` varchar(500) DEFAULT NULL COMMENT '来源页面',
                                    `accessed_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '访问时间',
                                    PRIMARY KEY (`id`),
                                    KEY `idx_content_id` (`content_id`),
                                    KEY `idx_user_id` (`user_id`),
                                    KEY `idx_accessed_at` (`accessed_at`),
                                    CONSTRAINT `fk_log_content` FOREIGN KEY (`content_id`) REFERENCES `clip_contents` (`id`) ON DELETE CASCADE,
                                    CONSTRAINT `fk_log_user` FOREIGN KEY (`user_id`) REFERENCES `clip_users` (`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Clip访问日志表';

-- Clip标签表（可选，用于更好的标签管理）
CREATE TABLE `clip_tags` (
                             `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT COMMENT '标签ID',
                             `name` varchar(50) NOT NULL COMMENT '标签名称',
                             `user_id` bigint(20) unsigned NOT NULL COMMENT '用户ID',
                             `usage_count` int(11) NOT NULL DEFAULT '0' COMMENT '使用次数',
                             `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
                             PRIMARY KEY (`id`),
                             UNIQUE KEY `uk_user_tag` (`user_id`, `name`),
                             KEY `idx_name` (`name`),
                             CONSTRAINT `fk_tag_user` FOREIGN KEY (`user_id`) REFERENCES `clip_users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='Clip标签表';