<?php
/**
 * SearchQueryAlgorithm
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
 * The version of the OpenAPI document: 0.11.0
 * Generated by: https://openapi-generator.tech
 * Generator version: 7.12.0
 */

/**
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

namespace Agnesoft\AgdbApi\Model;
use \Agnesoft\AgdbApi\ObjectSerializer;

/**
 * SearchQueryAlgorithm Class Doc Comment
 *
 * @category Class
 * @description Search algorithm to be used
 * @package  Agnesoft\AgdbApi
 * @author   OpenAPI Generator team
 * @link     https://openapi-generator.tech
 */
class SearchQueryAlgorithm
{
    /**
     * Possible values of this enum
     */
    public const BREADTH_FIRST = 'BreadthFirst';

    public const DEPTH_FIRST = 'DepthFirst';

    public const INDEX = 'Index';

    public const ELEMENTS = 'Elements';

    /**
     * Gets allowable values of the enum
     * @return string[]
     */
    public static function getAllowableEnumValues()
    {
        return [
            self::BREADTH_FIRST,
            self::DEPTH_FIRST,
            self::INDEX,
            self::ELEMENTS
        ];
    }
}


