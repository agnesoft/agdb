<?php

namespace Agnesoft\AgdbApi;

/**
 * The transport of the generated `AgdbApi` over native cURL: bodies are JSON
 * and the token is a bearer one. A response that is not a success throws an
 * `AgdbApiError`, a failed request throws with status 500.
 */
class CurlClient extends HttpClient
{
    public function __construct(private string $userAgent = "agdb_api") {}

    public function delete(string $uri, ?string $token): int
    {
        return $this->send("DELETE", $uri, null, $token)[0];
    }

    /** @return array{0: int, 1: mixed} */
    public function get(string $uri, ?string $token): array
    {
        return $this->send("GET", $uri, null, $token);
    }

    /** @return array{0: int, 1: mixed} */
    public function post(string $uri, mixed $json, ?string $token): array
    {
        return $this->send("POST", $uri, $json, $token);
    }

    public function put(string $uri, mixed $json, ?string $token): int
    {
        return $this->send("PUT", $uri, $json, $token)[0];
    }

    /**
     * @return array{0: int, 1: mixed}
     */
    private function send(
        string $method,
        string $uri,
        mixed $json,
        ?string $token
    ): array {
        $ch = curl_init($uri);

        if ($ch === false) {
            throw new AgdbApiError(500, "Failed to initialize cURL");
        }

        $headers = ["User-Agent: " . $this->userAgent];

        if ($token !== null) {
            $headers[] = "Authorization: Bearer " . $token;
        }

        if ($json !== null) {
            $body = json_encode($json, JSON_THROW_ON_ERROR);
            $headers[] = "Content-Type: application/json";
            $headers[] = "Content-Length: " . strlen($body);
            curl_setopt($ch, CURLOPT_POSTFIELDS, $body);
        }

        curl_setopt($ch, CURLOPT_CUSTOMREQUEST, $method);
        curl_setopt($ch, CURLOPT_HTTPHEADER, $headers);
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);

        $responseBody = curl_exec($ch);

        if ($responseBody === false) {
            $error = curl_error($ch);
            curl_close($ch);
            throw new AgdbApiError(500, $error);
        }

        /** @var int $status */
        $status = curl_getinfo($ch, CURLINFO_HTTP_CODE);
        curl_close($ch);

        if ($status < 200 || $status >= 300) {
            throw new AgdbApiError(
                $status,
                is_string($responseBody) ? $responseBody : ""
            );
        }

        $decoded =
            $responseBody === "" ? null : json_decode($responseBody, true);

        return [$status, $decoded];
    }
}
