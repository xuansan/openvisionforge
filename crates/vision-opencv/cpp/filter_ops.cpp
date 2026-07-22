#include "bridge_common.hpp"

static int odd_kernel(const std::string& value, int min_value = 3) {
    int kernel = std::stoi(value);
    if (kernel % 2 == 0) {
        kernel += 1;
    }
    return std::max(kernel, min_value);
}

static std::string op_blur(const std::vector<std::string>& args) {
    if (args.size() < 4) {
        fail("usage: blur <input> <output> <kernel>");
    }
    cv::Mat image = read_image(args[1]);
    cv::Mat out;
    int kernel = std::max(std::stoi(args[3]), 1);
    cv::blur(image, out, cv::Size(kernel, kernel));
    write_image(args[2], out);
    return metadata_json(out, args[2]);
}

static std::string op_gaussian(const std::vector<std::string>& args) {
    if (args.size() < 5) {
        fail("usage: gaussian <input> <output> <kernel> <sigma>");
    }
    cv::Mat image = read_image(args[1]);
    cv::Mat out;
    int kernel = odd_kernel(args[3]);
    double sigma = std::stod(args[4]);
    cv::GaussianBlur(image, out, cv::Size(kernel, kernel), sigma);
    write_image(args[2], out);
    return metadata_json(out, args[2]);
}

static std::string op_median(const std::vector<std::string>& args) {
    if (args.size() < 4) {
        fail("usage: median <input> <output> <kernel>");
    }
    cv::Mat image = read_image(args[1]);
    cv::Mat out;
    cv::medianBlur(image, out, odd_kernel(args[3]));
    write_image(args[2], out);
    return metadata_json(out, args[2]);
}

static std::string op_sharpen(const std::vector<std::string>& args) {
    if (args.size() < 3) {
        fail("usage: sharpen <input> <output>");
    }
    cv::Mat image = read_image(args[1]);
    cv::Mat out;
    cv::Mat kernel = (cv::Mat_<float>(3, 3) << 0, -1, 0, -1, 5, -1, 0, -1, 0);
    cv::filter2D(image, out, image.depth(), kernel);
    write_image(args[2], out);
    return metadata_json(out, args[2]);
}
