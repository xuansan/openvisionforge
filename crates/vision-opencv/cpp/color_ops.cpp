#include "bridge_common.hpp"

static cv::Mat to_gray_mat(const cv::Mat& image) {
    if (image.channels() == 1) {
        return image.clone();
    }
    cv::Mat gray;
    if (image.channels() == 3) {
        cv::cvtColor(image, gray, cv::COLOR_BGR2GRAY);
    } else if (image.channels() == 4) {
        cv::cvtColor(image, gray, cv::COLOR_BGRA2GRAY);
    } else {
        fail("unsupported channel count for grayscale conversion");
    }
    return gray;
}

static std::string op_gray(const std::vector<std::string>& args) {
    if (args.size() < 3) {
        fail("usage: gray <input> <output>");
    }
    cv::Mat image = read_image(args[1]);
    cv::Mat gray = to_gray_mat(image);
    write_image(args[2], gray);
    return metadata_json(gray, args[2]);
}

static std::string op_color_convert(const std::vector<std::string>& args) {
    if (args.size() < 4) {
        fail("usage: color_convert <input> <output> <mode>");
    }
    cv::Mat image = read_image(args[1]);
    cv::Mat converted;
    const std::string& mode = args[3];
    if (mode == "bgr_to_hsv") {
        cv::cvtColor(image, converted, cv::COLOR_BGR2HSV);
    } else if (mode == "bgr_to_lab") {
        cv::cvtColor(image, converted, cv::COLOR_BGR2Lab);
    } else if (mode == "bgr_to_rgb") {
        cv::cvtColor(image, converted, cv::COLOR_BGR2RGB);
    } else if (mode == "rgb_to_bgr") {
        cv::cvtColor(image, converted, cv::COLOR_RGB2BGR);
    } else if (mode == "bgr_to_ycrcb") {
        cv::cvtColor(image, converted, cv::COLOR_BGR2YCrCb);
    } else {
        fail("unsupported color convert mode: " + mode);
    }
    write_image(args[2], converted);
    return metadata_json(converted, args[2]);
}
