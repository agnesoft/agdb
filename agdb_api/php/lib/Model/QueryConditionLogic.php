<?php
/**
 * QueryConditionLogic
 *
 * PHP version 7.4
 *
 * @category Class
 * @package  Agnesoft\AgdbApi
 * @author   OpenAPI Generator team
 * @link     https://openapi-generator.tech
 */

/**
 * agdb_server
 *
 * Agnesoft Graph Database Server
 *
 * The version of the OpenAPI document: 0.9.2
 * Generated by: https://openapi-generator.tech
 * Generator version: 7.9.0
 */

/**
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

namespace Agnesoft\AgdbApi\Model;
use \Agnesoft\AgdbApi\ObjectSerializer;

/**
 * QueryConditionLogic Class Doc Comment
 *
 * @category Class
 * @description Logical operator for query conditions
 * @package  Agnesoft\AgdbApi
 * @author   OpenAPI Generator team
 * @link     https://openapi-generator.tech
 */
class QueryConditionLogic
{
    /**
     * Possible values of this enum
     */
    public const _AND = 'And';

    public const _OR = 'Or';

    /**
     * Gets allowable values of the enum
     * @return string[]
     */
    public static function getAllowableEnumValues()
    {
        return [
            self::_AND,
            self::_OR
        ];
    }
}


