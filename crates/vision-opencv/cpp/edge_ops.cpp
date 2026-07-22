#include "bridge_common.hpp"

static std::string op_canny(const std::vector<std::string>& args) {
    if (args.size() < 5) {
        fail("usage: canny <input> <output> <low> <high>");
    }
    cv::Mat gray = to_gray_mat(read_image(args[1]));
    cv::Mat edges;
    cv::Canny(gray, edges, std::stod(args[3]), std::stod(args[4]));
    write_image(args[2], edges);
    return metadata_json(edges, args[2]);
}
