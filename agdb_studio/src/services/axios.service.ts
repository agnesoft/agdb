import axios from "axios";

const axiosInstance = axios.create({
    baseURL: import.meta.env.VUE_APP_API_URL,
    headers: {
        post: {
            "Content-Type": "application/json",
        },
        // TODO: set token: "atoken": `Bearer ${getToken()}`,
    },
});

axiosInstance.interceptors.response.use(
    (response) => {
        return response.data;
    },
    (error: any) => {
        if (!error.response) {
            return Promise.reject(error);
        }
        if (error.response.status === 401) {
            // TODO: logout
            return Promise.reject(error);
        }
        if (error.response.status === 302) {
            // TODO: handle redirect
            return Promise.reject(error);
        }
        if (error.response.status === 404) {
            // TODO: redirect to 404 page
            return Promise.reject(error);
        }
        if (error.response.status === 500) {
            return Promise.reject(error);
        }
        const data = error.response.data;
        const errorData: string = (data && data.message) || "Unknown error";
        return Promise.reject(errorData);
    },
);

export default axiosInstance;
