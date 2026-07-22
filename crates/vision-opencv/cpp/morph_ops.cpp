#include "bridge_common.hpp"

static std::string op_morph(const std::vector<std::string>& args) {
    if (args.size() < 6) {
        fail("usage: morph <input> <output> <op> <kernel> <iterations>");
    }
    cv::Mat image = read_image(args[1]);
    cv::Mat out;
    int kernel_size = std::max(std::stoi(args[4]), 1);
    int iterations = std::max(std::stoi(args[5]), 1);
    cv::Mat kernel = cv::getStructuringElement(
        cv::MORPH_RECT,
        cv::Size(kernel_size, kernel_size)
    );

    const std::string& op = args[3];
    if (op == "erode") {
        cv::erode(image, out, kernel, cv::Point(-1, -1), iterations);
    } else if (op == "dilate") {
        cv::dilate(image, out, kernel, cv::Point(-1, -1), iterations);
    } else {
        int morph_op;
        if (op == "open") {
            morph_op = cv::MORPH_OPEN;
        } else if (op == "close") {
            morph_op = cv::MORPH_CLOSE;
        } else if (op == "gradient") {
            morph_op = cv::MORPH_GRADIENT;
        } else if (op == "tophat") {
            morph_op = cv::MORPH_TOPHAT;
        } else if (op == "blackhat") {
            morph_op = cv::MORPH_BLACKHAT;
        } else {
            fail("unsupported morph op: " + op);
        }
        cv::morphologyEx(image, out, morph_op, kernel, cv::Point(-1, -1), iterations);
    }

    write_image(args[2], out);
    return metadata_json(out, args[2]);
}
