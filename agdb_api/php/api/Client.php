<?php

namespace Agnesoft\Agdb;

class Client extends \Agnesoft\Agdb\Runtime\Client\Client
{
    /**
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminDbListUnauthorizedException
     *
     * @return null|\Agnesoft\Agdb\Model\ServerDatabase[]|\Psr\Http\Message\ResponseInterface
     */
    public function adminDbList(string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminDbList(), $fetch);
    }
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     * @param array $queryParameters {
     *     @var string $db_type 
     * }
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminDbAddUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminDbAddNotFoundException
     * @throws \Agnesoft\Agdb\Exception\AdminDbAddCustom465Exception
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function adminDbAdd(string $owner, string $db, array $queryParameters = [], string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminDbAdd($owner, $db, $queryParameters), $fetch);
    }
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminDbAuditUnauthorizedException
     *
     * @return null|\Agnesoft\Agdb\Model\QueryAudit[]|\Psr\Http\Message\ResponseInterface
     */
    public function adminDbAudit(string $owner, string $db, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminDbAudit($owner, $db), $fetch);
    }
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminDbBackupUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminDbBackupForbiddenException
     * @throws \Agnesoft\Agdb\Exception\AdminDbBackupNotFoundException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function adminDbBackup(string $owner, string $db, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminDbBackup($owner, $db), $fetch);
    }
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param array $queryParameters {
     *     @var string $new_name 
     * }
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminDbCopyUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminDbCopyNotFoundException
     * @throws \Agnesoft\Agdb\Exception\AdminDbCopyCustom465Exception
     * @throws \Agnesoft\Agdb\Exception\AdminDbCopyCustom467Exception
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function adminDbCopy(string $owner, string $db, array $queryParameters = [], string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminDbCopy($owner, $db, $queryParameters), $fetch);
    }
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminDbDeleteUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminDbDeleteNotFoundException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function adminDbDelete(string $owner, string $db, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminDbDelete($owner, $db), $fetch);
    }
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param array[] $requestBody 
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminDbExecUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminDbExecForbiddenException
     * @throws \Agnesoft\Agdb\Exception\AdminDbExecNotFoundException
     *
     * @return null|\Agnesoft\Agdb\Model\QueryResult[]|\Psr\Http\Message\ResponseInterface
     */
    public function adminDbExec(string $owner, string $db, array $requestBody, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminDbExec($owner, $db, $requestBody), $fetch);
    }
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminDbOptimizeUnauthorizedException
     *
     * @return null|\Agnesoft\Agdb\Model\ServerDatabase|\Psr\Http\Message\ResponseInterface
     */
    public function adminDbOptimize(string $owner, string $db, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminDbOptimize($owner, $db), $fetch);
    }
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminDbRemoveUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminDbRemoveNotFoundException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function adminDbRemove(string $owner, string $db, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminDbRemove($owner, $db), $fetch);
    }
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param array $queryParameters {
     *     @var string $new_name 
     * }
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminDbRenameUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminDbRenameNotFoundException
     * @throws \Agnesoft\Agdb\Exception\AdminDbRenameCustom465Exception
     * @throws \Agnesoft\Agdb\Exception\AdminDbRenameCustom467Exception
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function adminDbRename(string $owner, string $db, array $queryParameters = [], string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminDbRename($owner, $db, $queryParameters), $fetch);
    }
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminDbUserListUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminDbUserListNotFoundException
     *
     * @return null|\Agnesoft\Agdb\Model\DbUser[]|\Psr\Http\Message\ResponseInterface
     */
    public function adminDbUserList(string $owner, string $db, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminDbUserList($owner, $db), $fetch);
    }
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param string $username user name
     * @param array $queryParameters {
     *     @var string $db_role 
     * }
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminDbUserAddUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminDbUserAddForbiddenException
     * @throws \Agnesoft\Agdb\Exception\AdminDbUserAddNotFoundException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function adminDbUserAdd(string $owner, string $db, string $username, array $queryParameters = [], string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminDbUserAdd($owner, $db, $username, $queryParameters), $fetch);
    }
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param string $username user name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminDbUserRemoveUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminDbUserRemoveForbiddenException
     * @throws \Agnesoft\Agdb\Exception\AdminDbUserRemoveNotFoundException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function adminDbUserRemove(string $owner, string $db, string $username, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminDbUserRemove($owner, $db, $username), $fetch);
    }
    /**
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminShutdownUnauthorizedException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function adminShutdown(string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminShutdown(), $fetch);
    }
    /**
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminUserListUnauthorizedException
     *
     * @return null|\Agnesoft\Agdb\Model\UserStatus[]|\Psr\Http\Message\ResponseInterface
     */
    public function adminUserList(string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminUserList(), $fetch);
    }
    /**
     * 
     *
     * @param string $username desired user name
     * @param \Agnesoft\Agdb\Model\UserCredentials $requestBody 
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminUserAddUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminUserAddCustom461Exception
     * @throws \Agnesoft\Agdb\Exception\AdminUserAddCustom462Exception
     * @throws \Agnesoft\Agdb\Exception\AdminUserAddCustom463Exception
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function adminUserAdd(string $username, \Agnesoft\Agdb\Model\UserCredentials $requestBody, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminUserAdd($username, $requestBody), $fetch);
    }
    /**
     * 
     *
     * @param string $username user name
     * @param \Agnesoft\Agdb\Model\UserCredentials $requestBody 
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminUserChangePasswordUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminUserChangePasswordCustom461Exception
     * @throws \Agnesoft\Agdb\Exception\AdminUserChangePasswordCustom464Exception
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function adminUserChangePassword(string $username, \Agnesoft\Agdb\Model\UserCredentials $requestBody, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminUserChangePassword($username, $requestBody), $fetch);
    }
    /**
     * 
     *
     * @param string $username user name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminUserRemoveUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminUserRemoveNotFoundException
     *
     * @return null|\Agnesoft\Agdb\Model\UserStatus[]|\Psr\Http\Message\ResponseInterface
     */
    public function adminUserRemove(string $username, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminUserRemove($username), $fetch);
    }
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\AdminDbRestoreUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminDbRestoreNotFoundException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function adminDbRestore(string $owner, string $db, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\AdminDbRestore($owner, $db), $fetch);
    }
    /**
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\DbListUnauthorizedException
     *
     * @return null|\Agnesoft\Agdb\Model\ServerDatabase[]|\Psr\Http\Message\ResponseInterface
     */
    public function dbList(string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\DbList(), $fetch);
    }
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     * @param array $queryParameters {
     *     @var string $db_type 
     * }
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\DbAddUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbAddForbiddenException
     * @throws \Agnesoft\Agdb\Exception\DbAddCustom465Exception
     * @throws \Agnesoft\Agdb\Exception\DbAddCustom467Exception
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function dbAdd(string $owner, string $db, array $queryParameters = [], string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\DbAdd($owner, $db, $queryParameters), $fetch);
    }
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\DbAuditUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbAuditNotFoundException
     *
     * @return null|\Agnesoft\Agdb\Model\QueryAudit[]|\Psr\Http\Message\ResponseInterface
     */
    public function dbAudit(string $owner, string $db, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\DbAudit($owner, $db), $fetch);
    }
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\DbBackupUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbBackupForbiddenException
     * @throws \Agnesoft\Agdb\Exception\DbBackupNotFoundException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function dbBackup(string $owner, string $db, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\DbBackup($owner, $db), $fetch);
    }
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     * @param array $queryParameters {
     *     @var string $resource 
     * }
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\DbClearUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbClearForbiddenException
     * @throws \Agnesoft\Agdb\Exception\DbClearNotFoundException
     *
     * @return null|\Agnesoft\Agdb\Model\ServerDatabase|\Psr\Http\Message\ResponseInterface
     */
    public function dbClear(string $owner, string $db, array $queryParameters = [], string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\DbClear($owner, $db, $queryParameters), $fetch);
    }
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param array $queryParameters {
     *     @var string $new_name 
     * }
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\DbCopyUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbCopyForbiddenException
     * @throws \Agnesoft\Agdb\Exception\DbCopyNotFoundException
     * @throws \Agnesoft\Agdb\Exception\DbCopyCustom465Exception
     * @throws \Agnesoft\Agdb\Exception\DbCopyCustom467Exception
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function dbCopy(string $owner, string $db, array $queryParameters = [], string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\DbCopy($owner, $db, $queryParameters), $fetch);
    }
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\DbDeleteUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbDeleteForbiddenException
     * @throws \Agnesoft\Agdb\Exception\DbDeleteNotFoundException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function dbDelete(string $owner, string $db, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\DbDelete($owner, $db), $fetch);
    }
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param array[] $requestBody 
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\DbExecUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbExecForbiddenException
     * @throws \Agnesoft\Agdb\Exception\DbExecNotFoundException
     *
     * @return null|\Agnesoft\Agdb\Model\QueryResult[]|\Psr\Http\Message\ResponseInterface
     */
    public function dbExec(string $owner, string $db, array $requestBody, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\DbExec($owner, $db, $requestBody), $fetch);
    }
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\DbOptimizeUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbOptimizeForbiddenException
     *
     * @return null|\Agnesoft\Agdb\Model\ServerDatabase|\Psr\Http\Message\ResponseInterface
     */
    public function dbOptimize(string $owner, string $db, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\DbOptimize($owner, $db), $fetch);
    }
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\DbRemoveUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbRemoveForbiddenException
     * @throws \Agnesoft\Agdb\Exception\DbRemoveNotFoundException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function dbRemove(string $owner, string $db, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\DbRemove($owner, $db), $fetch);
    }
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param array $queryParameters {
     *     @var string $new_name 
     * }
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\DbRenameUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbRenameForbiddenException
     * @throws \Agnesoft\Agdb\Exception\DbRenameNotFoundException
     * @throws \Agnesoft\Agdb\Exception\DbRenameCustom465Exception
     * @throws \Agnesoft\Agdb\Exception\DbRenameCustom467Exception
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function dbRename(string $owner, string $db, array $queryParameters = [], string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\DbRename($owner, $db, $queryParameters), $fetch);
    }
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\DbRestoreUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbRestoreForbiddenException
     * @throws \Agnesoft\Agdb\Exception\DbRestoreNotFoundException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function dbRestore(string $owner, string $db, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\DbRestore($owner, $db), $fetch);
    }
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\DbUserListUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbUserListNotFoundException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function dbUserList(string $owner, string $db, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\DbUserList($owner, $db), $fetch);
    }
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param string $username user name
     * @param array $queryParameters {
     *     @var string $db_role 
     * }
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\DbUserAddUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbUserAddForbiddenException
     * @throws \Agnesoft\Agdb\Exception\DbUserAddNotFoundException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function dbUserAdd(string $owner, string $db, string $username, array $queryParameters = [], string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\DbUserAdd($owner, $db, $username, $queryParameters), $fetch);
    }
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param string $username user name
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\DbUserRemoveUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbUserRemoveForbiddenException
     * @throws \Agnesoft\Agdb\Exception\DbUserRemoveNotFoundException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function dbUserRemove(string $owner, string $db, string $username, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\DbUserRemove($owner, $db, $username), $fetch);
    }
    /**
     * 
     *
     * @param bool $cluster get cluster status
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     *
     * @return null|\Agnesoft\Agdb\Model\ClusterStatus[]|\Psr\Http\Message\ResponseInterface
     */
    public function status(bool $cluster, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\Status($cluster), $fetch);
    }
    /**
     * 
     *
     * @param \Agnesoft\Agdb\Model\ChangePassword $requestBody 
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\UserChangePasswordUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\UserChangePasswordCustom461Exception
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function userChangePassword(\Agnesoft\Agdb\Model\ChangePassword $requestBody, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\UserChangePassword($requestBody), $fetch);
    }
    /**
     * 
     *
     * @param \Agnesoft\Agdb\Model\UserLogin $requestBody 
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\UserLoginUnauthorizedException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function userLogin(\Agnesoft\Agdb\Model\UserLogin $requestBody, string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\UserLogin($requestBody), $fetch);
    }
    /**
     * @param string $fetch Fetch mode to use (can be OBJECT or RESPONSE)
     * @throws \Agnesoft\Agdb\Exception\UserLogoutUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\UserLogoutNotFoundException
     *
     * @return null|\Psr\Http\Message\ResponseInterface
     */
    public function userLogout(string $fetch = self::FETCH_OBJECT)
    {
        return $this->executeEndpoint(new \Agnesoft\Agdb\Endpoint\UserLogout(), $fetch);
    }
    public static function create($httpClient = null, array $additionalPlugins = [], array $additionalNormalizers = [])
    {
        if (null === $httpClient) {
            $httpClient = \Http\Discovery\Psr18ClientDiscovery::find();
            $plugins = [];
            $uri = \Http\Discovery\Psr17FactoryDiscovery::findUriFactory()->createUri('http://localhost:3000');
            $plugins[] = new \Http\Client\Common\Plugin\AddHostPlugin($uri);
            if (count($additionalPlugins) > 0) {
                $plugins = array_merge($plugins, $additionalPlugins);
            }
            $httpClient = new \Http\Client\Common\PluginClient($httpClient, $plugins);
        }
        $requestFactory = \Http\Discovery\Psr17FactoryDiscovery::findRequestFactory();
        $streamFactory = \Http\Discovery\Psr17FactoryDiscovery::findStreamFactory();
        $normalizers = [new \Symfony\Component\Serializer\Normalizer\ArrayDenormalizer(), new \Agnesoft\Agdb\Normalizer\JaneObjectNormalizer()];
        if (count($additionalNormalizers) > 0) {
            $normalizers = array_merge($normalizers, $additionalNormalizers);
        }
        $serializer = new \Symfony\Component\Serializer\Serializer($normalizers, [new \Symfony\Component\Serializer\Encoder\JsonEncoder(new \Symfony\Component\Serializer\Encoder\JsonEncode(), new \Symfony\Component\Serializer\Encoder\JsonDecode(['json_decode_associative' => true]))]);
        return new static($httpClient, $requestFactory, $serializer, $streamFactory);
    }
}