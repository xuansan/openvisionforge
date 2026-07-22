#include "bridge_common.hpp"

static std::string op_threshold(const std::vector<std::string>& args) {
    if (args.size() < 4) {
        fail("usage: threshold <input> <output> <threshold>");
    }
    cv::Mat gray = to_gray_mat(read_image(args[1]));
    cv::Mat binary;
    double threshold = std::stod(args[3]);
    cv::threshold(gray, binary, threshold, 255.0, cv::THRESH_BINARY);
    write_image(args[2], binary);
    return metadata_json(binary, args[2]);
}

static std::string op_threshold_otsu(const std::vector<std::string>& args) {
    if (args.size() < 3) {
        fail("usage: threshold_otsu <input> <output>");
    }
    cv::Mat gray = to_gray_mat(read_image(args[1]));
    cv::Mat binary;
    cv::threshold(gray, binary, 0.0, 255.0, cv::THRESH_BINARY | cv::THRESH_OTSU);
    write_image(args[2], binary);
    return metadata_json(binary, args[2]);
}

static std::string op_threshold_triangle(const std::vector<std::string>& args) {
    if (args.size() < 3) {
        fail("usage: threshold_triangle <input> <output>");
    }
    cv::Mat gray = to_gray_mat(read_image(args[1]));
    cv::Mat binary;
    cv::threshold(gray, binary, 0.0, 255.0, cv::THRESH_BINARY | cv::THRESH_TRIANGLE);
    write_image(args[2], binary);
    return metadata_json(binary, args[2]);
}

static std::string op_threshold_adaptive(const std::vector<std::string>& args) {
    if (args.size() < 6) {
        fail("usage: threshold_adaptive <input> <output> <mean|gaussian> <block_size> <c>");
    }
    cv::Mat gray = to_gray_mat(read_image(args[1]));
    cv::Mat binary;
    int method = args[3] == "gaussian" ? cv::ADAPTIVE_THRESH_GAUSSIAN_C : cv::ADAPTIVE_THRESH_MEAN_C;
    int block_size = std::stoi(args[4]);
    if (block_size % 2 == 0) {
        block_size += 1;
    }
    if (block_size < 3) {
        block_size = 3;
    }
    double c = std::stod(args[5]);
    cv::adaptiveThreshold(gray, binary, 255.0, method, cv::THRESH_BINARY, block_size, c);
    write_image(args[2], binary);
    return metadata_json(binary, args[2]);
}

static std::string op_threshold_range(const std::vector<std::string>& args) {
    if (args.size() < 5) {
        fail("usage: threshold_range <input> <output> <low> <high>");
    }
    cv::Mat gray = to_gray_mat(read_image(args[1]));
    cv::Mat binary;
    cv::inRange(gray, std::stod(args[3]), std::stod(args[4]), binary);
    write_image(args[2], binary);
    return metadata_json(binary, args[2]);
}
