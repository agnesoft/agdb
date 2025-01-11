<?php
/**
 * QueryConditionModifier
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
 * The version of the OpenAPI document: 0.10.0
 * Generated by: https://openapi-generator.tech
 * Generator version: 7.10.0
 */

/**
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

namespace Agnesoft\AgdbApi\Model;
use \Agnesoft\AgdbApi\ObjectSerializer;

/**
 * QueryConditionModifier Class Doc Comment
 *
 * @category Class
 * @description Query condition modifier
 * @package  Agnesoft\AgdbApi
 * @author   OpenAPI Generator team
 * @link     https://openapi-generator.tech
 */
class QueryConditionModifier
{
    /**
     * Possible values of this enum
     */
    public const NONE = 'None';

    public const BEYOND = 'Beyond';

    public const NOT = 'Not';

    public const NOT_BEYOND = 'NotBeyond';

    /**
     * Gets allowable values of the enum
     * @return string[]
     */
    public static function getAllowableEnumValues()
    {
        return [
            self::NONE,
            self::BEYOND,
            self::NOT,
            self::NOT_BEYOND
        ];
    }
}


